//! Paint scheduling for CSS positioned content and integer stacking contexts.
//! Input children are already in forward CSS paint-tree order. An auto-z
//! positioned node does not seal its positioned descendants into a context.
//! Clip handles describe ancestor scopes; executing them must not repaint the
//! ancestor's background. Integer levels are atomic, unlike auto positioning.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PaintTree<D, C> {
    pub before: Vec<D>,
    pub children: Vec<Self>,
    pub after: Vec<D>,
    pub positioned: bool,
    /// None is auto; an integer seals descendants into a stacking context.
    pub z_index: Option<i32>,
    /// Validated CSS alpha; values below one establish an isolated context.
    pub opacity: f32,
    /// Compiler occurrence order is CSS order-modified preorder, independent
    /// of the ordinary reverse-flex paint traversal. None retains input order.
    pub stacking_order: Option<usize>,
    pub clip: Option<C>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum OpacityBoundary {
    Begin(f32),
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PaintGroup<D, C> {
    pub opacity_boundary: Option<OpacityBoundary>,
    pub ancestor_clips: Vec<C>,
    pub draws: Vec<D>,
}

fn is_context<D, C>(node: &PaintTree<D, C>) -> bool {
    node.z_index.is_some() || node.opacity < 1.0
}

pub(super) fn plan<D: Clone, C: Clone>(roots: &[PaintTree<D, C>]) -> Vec<PaintGroup<D, C>> {
    fn has_context<D, C>(node: &PaintTree<D, C>) -> bool {
        is_context(node) || node.children.iter().any(has_context)
    }
    if roots.iter().any(has_context) { return stacking_plan(roots); }
    fn ordinary<D: Clone, C: Clone>(node: &PaintTree<D, C>, output: &mut Vec<D>) {
        output.extend_from_slice(&node.before);
        for child in &node.children {
            if !child.positioned { ordinary(child, output); }
        }
        output.extend_from_slice(&node.after);
    }
    fn positioned<D: Clone, C: Clone>(
        node: &PaintTree<D, C>, clips: &mut Vec<C>, groups: &mut Vec<PaintGroup<D, C>>,
    ) {
        if node.positioned {
            let mut draws = Vec::new();
            ordinary(node, &mut draws);
            if !draws.is_empty() {
                groups.push(PaintGroup { opacity_boundary: None, ancestor_clips: clips.clone(), draws });
            }
        }
        // The node's own before/after commands manage its own clip. Only
        // descendants deferred out of that group need to reopen this scope.
        if let Some(clip) = &node.clip { clips.push(clip.clone()); }
        for child in &node.children { positioned(child, clips, groups); }
        if node.clip.is_some() { clips.pop(); }
    }
    let mut groups = Vec::new();
    let mut draws = Vec::new();
    for root in roots {
        if !root.positioned { ordinary(root, &mut draws); }
    }
    if !draws.is_empty() { groups.push(PaintGroup { opacity_boundary: None, ancestor_clips: Vec::new(), draws }); }
    let mut clips = Vec::new();
    for root in roots { positioned(root, &mut clips, &mut groups); }
    groups

}

fn stacking_plan<D: Clone, C: Clone>(roots: &[PaintTree<D, C>]) -> Vec<PaintGroup<D, C>> {
    fn ordinary<D: Clone, C>(node: &PaintTree<D, C>, draws: &mut Vec<D>) {
        draws.extend_from_slice(&node.before);
        for child in &node.children {
            if !child.positioned && !is_context(child) { ordinary(child, draws); }
        }
        draws.extend_from_slice(&node.after);
    }
    fn collect<'a, D, C: Clone>(
        node: &'a PaintTree<D, C>, clips: &mut Vec<C>,
        entries: &mut Vec<(&'a PaintTree<D, C>, Vec<C>)>,
    ) {
        if is_context(node) || node.positioned {
            entries.push((node, clips.clone()));
        }
        // Integer contexts are atomic. Auto positioned descendants still
        // participate in this context, including negative descendants.
        if is_context(node) { return; }
        if let Some(clip) = &node.clip { clips.push(clip.clone()); }
        for child in &node.children { collect(child, clips, entries); }
        if node.clip.is_some() { clips.pop(); }
    }
    fn emit<D, C: Clone>(draws: Vec<D>, clips: &[C], output: &mut Vec<PaintGroup<D, C>>) {
        if !draws.is_empty() { output.push(PaintGroup { opacity_boundary: None, ancestor_clips: clips.to_vec(), draws }); }
    }
    fn entry<D: Clone, C: Clone>(node: &PaintTree<D, C>, clips: &[C], output: &mut Vec<PaintGroup<D, C>>) {
        if !is_context(node) {
            let mut draws = Vec::new();
            ordinary(node, &mut draws);
            emit(draws, clips, output);
            return;
        }
        if node.opacity < 1.0 {
            output.push(PaintGroup { opacity_boundary: Some(OpacityBoundary::Begin(node.opacity)),
                ancestor_clips: Vec::new(), draws: Vec::new() });
        }
        // Paint the context background once, closing its proxy clip before
        // deferred groups reopen ancestor clips. Runtime `after` is the
        // balancing layout occurrence, not foreground content.
        let mut background = node.before.clone();
        background.extend_from_slice(&node.after);
        emit(background, clips, output);
        let mut descendants = clips.to_vec();
        if let Some(clip) = &node.clip { descendants.push(clip.clone()); }
        context(&node.children, &descendants, output);
        if node.opacity < 1.0 {
            output.push(PaintGroup { opacity_boundary: Some(OpacityBoundary::End),
                ancestor_clips: Vec::new(), draws: Vec::new() });
        }
    }
    fn context<D: Clone, C: Clone>(roots: &[PaintTree<D, C>], clips: &[C], output: &mut Vec<PaintGroup<D, C>>) {
        let mut entries = Vec::new();
        let mut ancestors = clips.to_vec();
        for root in roots { collect(root, &mut ancestors, &mut entries); }
        // Stable sorting retains incoming CSS order for equal levels, including
        // auto and zero. Flex direction must not reverse this order.
        entries.sort_by_key(|(node, _)| (node.z_index.unwrap_or(0), node.stacking_order));
        for (node, ancestors) in &entries {
            if node.z_index.unwrap_or(0) < 0 { entry(node, ancestors, output); }
        }
        let mut draws = Vec::new();
        for root in roots {
            if !root.positioned && !is_context(root) { ordinary(root, &mut draws); }
        }
        emit(draws, clips, output);
        for (node, ancestors) in &entries {
            if node.z_index.unwrap_or(0) >= 0 { entry(node, ancestors, output); }
        }
    }
    let mut output = Vec::new();
    context(roots, &[], &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(name: &'static str, positioned: bool, clip: Option<u8>, children: Vec<PaintTree<&'static str, u8>>) -> PaintTree<&'static str, u8> {
        PaintTree { before: vec![name], after: vec![], children, positioned, z_index: None, opacity: 1.0, stacking_order: None, clip }
    }
    fn names(groups: &[PaintGroup<&'static str, u8>]) -> Vec<&'static str> {
        groups.iter().flat_map(|g| g.draws.iter().copied()).collect()
    }
    fn level(mut tree: PaintTree<&'static str, u8>, z: i32) -> PaintTree<&'static str, u8> {
        tree.z_index = Some(z);
        tree
    }
    #[test]
    fn opacity_isolates_negative_and_positive_descendants_but_one_does_not() {
        for alpha in [0.0, 0.5, 1.0] {
            let mut parent = node("parent", false, None, vec![
                level(node("negative", true, None, vec![]), -2),
                node("ordinary", false, None, vec![]),
                level(node("positive", true, None, vec![]), 2),
            ]);
            parent.opacity = alpha;
            let groups = plan(&[parent, level(node("outside", true, None, vec![]), 1)]);
            if alpha < 1.0 {
                assert_eq!(names(&groups), ["parent", "negative", "ordinary", "positive", "outside"]);
                assert_eq!(groups.first().unwrap().opacity_boundary, Some(OpacityBoundary::Begin(alpha)));
                assert_eq!(groups[groups.len()-2].opacity_boundary, Some(OpacityBoundary::End));
            } else {
                assert_eq!(names(&groups), ["negative", "parent", "ordinary", "outside", "positive"]);
                assert!(groups.iter().all(|g| g.opacity_boundary.is_none()));
            }
        }
    }

    #[test]
    fn nested_opacity_boundaries_enclose_background_and_deferred_clipped_content() {
        let mut inner = node("inner", false, Some(2), vec![level(node("child", true, None, vec![]), 4)]);
        inner.opacity = 0.25;
        let mut outer = node("outer", false, Some(1), vec![inner]);
        outer.opacity = 0.5;
        let groups = plan(&[outer]);
        assert_eq!(names(&groups), ["outer", "inner", "child"]);
        assert_eq!(groups.iter().filter_map(|g| g.opacity_boundary.clone()).collect::<Vec<_>>(),
            vec![OpacityBoundary::Begin(0.5), OpacityBoundary::Begin(0.25), OpacityBoundary::End, OpacityBoundary::End]);
        assert_eq!(groups.iter().find(|g| g.draws == ["child"]).unwrap().ancestor_clips, [1, 2]);
    }

    #[test]
    fn auto_descendant_escapes_but_zero_context_is_atomic() {
        let inner = level(node("inner", true, None, vec![]), 3);
        let parent = node("parent", true, None, vec![inner]);
        let sibling = level(node("sibling", true, None, vec![]), 1);
        assert_eq!(names(&plan(&[parent.clone(), sibling.clone()])), ["parent", "sibling", "inner"]);
        assert_eq!(names(&plan(&[level(parent, 0), sibling])), ["parent", "inner", "sibling"]);
    }
    #[test]
    fn negative_context_is_above_own_background_but_below_ordinary_content() {
        let children = vec![node("ordinary", false, None, vec![]), level(node("negative", true, None, vec![]), -1)];
        assert_eq!(names(&plan(&[level(node("background", true, None, children), 0)])), ["background", "negative", "ordinary"]);
    }
    #[test]
    fn negative_descendant_of_auto_ancestor_precedes_that_ancestor() {
        let parent = node("auto", true, None, vec![level(node("negative", true, None, vec![]), -1)]);
        assert_eq!(names(&plan(&[node("ordinary", false, None, vec![]), parent])), ["negative", "ordinary", "auto"]);
    }
    #[test]
    fn static_flex_context_seals_high_descendant() {
        let parent = level(node("static-context", false, None, vec![level(node("inner", true, None, vec![]), 100)]), 0);
        assert_eq!(names(&plan(&[parent, level(node("sibling", true, None, vec![]), 1)])), ["static-context", "inner", "sibling"]);
    }
    #[test]
    fn signed_levels_sort_stably_without_overflow() {
        let roots = vec![level(node("high", true, None, vec![]), i32::MAX), level(node("equal-a", true, None, vec![]), 1), node("auto", true, None, vec![]), level(node("low", true, None, vec![]), i32::MIN), level(node("zero", true, None, vec![]), 0), level(node("equal-b", true, None, vec![]), 1)];
        assert_eq!(names(&plan(&roots)), ["low", "auto", "zero", "equal-a", "equal-b", "high"]);
    }
    #[test]
    fn stacking_ties_use_occurrence_order_without_reordering_ordinary_paint() {
        let mut later = level(node("later-context", true, None, vec![]), 0);
        later.stacking_order = Some(20);
        let mut earlier = node("earlier-auto", true, None, vec![]);
        earlier.stacking_order = Some(10);
        let trees = [later, node("ordinary-b", false, None, vec![]), earlier, node("ordinary-a", false, None, vec![])];
        assert_eq!(names(&plan(&trees)), ["ordinary-b", "ordinary-a", "earlier-auto", "later-context"]);
    }
    #[test]
    fn context_clips_are_reopened_without_repainting_background() {
        let mut context = level(node("open", true, Some(2), vec![level(node("negative", true, None, vec![]), -1), node("ordinary", false, None, vec![])]), 0);
        context.after = vec!["close"];
        let trees = [node("ancestor", false, Some(1), vec![context])];
        let groups = plan(&trees);
        assert_eq!(names(&groups), ["ancestor", "open", "close", "negative", "ordinary"]);
        assert_eq!(groups.iter().map(|g| g.ancestor_clips.clone()).collect::<Vec<_>>(), [vec![], vec![1], vec![1, 2], vec![1, 2]]);
    }
    #[test]
    fn positioned_sibling_paints_after_ordinary_siblings() {
        let trees=vec![node("root",false,None,vec![node("positioned",true,None,vec![]),node("ordinary",false,None,vec![])])];
        assert_eq!(names(&plan(&trees)),["root","ordinary","positioned"]);
    }
    #[test]
    fn positioned_descendant_escapes_ordinary_group_without_repainting_ancestor() {
        let trees=vec![node("root",false,None,vec![node("a",false,None,vec![node("inner",true,None,vec![])]),node("b",false,None,vec![])])];
        assert_eq!(names(&plan(&trees)),["root","a","b","inner"]);
    }
    #[test]
    fn auto_positioned_descendant_remains_before_later_positioned_sibling() {
        let trees=vec![node("root",false,None,vec![node("a",true,None,vec![node("inner",true,None,vec![])]),node("b",true,None,vec![])])];
        let groups=plan(&trees);
        assert_eq!(names(&groups),["root","a","inner","b"]);
        assert_eq!(groups.iter().map(|g|g.draws.len()).collect::<Vec<_>>(),[1,1,1,1]);
    }
    #[test]
    fn deferred_descendant_retains_all_ancestor_clips_but_not_its_own() {
        let trees=vec![node("root",false,Some(1),vec![node("a",true,Some(2),vec![node("inner",true,Some(3),vec![])])])];
        let groups=plan(&trees);
        assert_eq!(groups.iter().map(|g|g.ancestor_clips.clone()).collect::<Vec<_>>(),[vec![],vec![1],vec![1,2]]);
    }
    #[test]
    fn ordinary_scope_end_stays_before_deferred_paint() {
        let mut a=node("open-a",false,Some(2),vec![node("inner",true,None,vec![])]);a.after=vec!["close-a"];
        let trees=vec![node("root",false,None,vec![a,node("b",false,None,vec![])])];
        let groups=plan(&trees);
        assert_eq!(names(&groups),["root","open-a","close-a","b","inner"]);
        assert_eq!(groups[1].ancestor_clips,[2]);
    }
    #[test]
    fn input_order_is_preserved_within_each_phase() {
        let trees=vec![node("third",true,None,vec![]),node("second",false,None,vec![]),node("first",true,None,vec![])];
        assert_eq!(names(&plan(&trees)),["second","third","first"]);
    }
    #[test]
    fn unpositioned_tree_retains_complete_draw_sequence() {
        let mut a=node("a",false,Some(1),vec![node("b",false,None,vec![])]);a.after=vec!["end-a"];
        assert_eq!(plan(&[a]),vec![PaintGroup{opacity_boundary:None,ancestor_clips:vec![],draws:vec!["a","b","end-a"]}]);
    }
}

// Convert the existing backwards CSS drawable sequence to a forward tree.
// Layout proxies open groups; authored layout occurrences close them when drawn.
pub(super) fn runtime_tree(
    drawables: &[crate::mechanical_port::source::drawable::RuntimeDrawableOccurrence],
    root: &crate::CoreHandle,
    positioned: &std::collections::HashSet<crate::CoreHandle>,
    levels: &std::collections::HashMap<crate::CoreHandle, (usize, Option<i32>)>,
    opacities: &std::collections::HashMap<crate::CoreHandle, f32>,
) -> Vec<PaintTree<crate::mechanical_port::source::drawable::RuntimeDrawableOccurrence, crate::CoreHandle>> {
    use crate::mechanical_port::source::drawable::RuntimeDrawableOccurrence as Occurrence;
    fn belongs(drawable: &Occurrence, layout: &crate::CoreHandle, root: &crate::CoreHandle) -> bool {
        let mut current = Some(match drawable {
            Occurrence::Authored(owner) => owner.clone(),
            Occurrence::RuntimeProxy(proxy) => proxy.borrow().hittable_component(),
        });
        while let Some(owner) = current {
            if owner == *layout { return true; }
            if owner == *root { return false; }
            current = owner.with(|o| o.component_parent_handle()).flatten();
        }
        false
    }
    let mut nodes = Vec::new();
    let mut index = 0;
    while index < drawables.len() {
        let start = index;
        let layout = drawables[index].authored_handle().filter(|owner|
            owner.with(|o| o.as_layout_component().is_some()).unwrap_or(false));
        index += 1;
        if let Some(layout) = layout {
            while index < drawables.len() && belongs(&drawables[index], &layout, root) { index += 1; }
            let has_proxy = index > start + 1 && matches!(&drawables[index-1], Occurrence::RuntimeProxy(proxy) if proxy.borrow().hittable_component() == layout);
            let end = index - usize::from(has_proxy);
            nodes.push(PaintTree {
                // CSS plans outlive clip changes. An initially plain container
                // still needs a proxy to open a clip enabled after planning.
                before: if has_proxy { vec![drawables[index-1].clone()] } else {
                    layout.with_mut(|o| o.as_layout_component_mut().and_then(|l| l.proxy()))
                        .flatten().into_iter().collect()
                },
                children: runtime_tree(&drawables[start+1..end], root, positioned, levels, opacities),
                after: vec![drawables[start].clone()],
                positioned: positioned.contains(&layout),
                opacity: opacities.get(&layout).copied().unwrap_or(1.0),
                z_index: levels.get(&layout).and_then(|(_, level)| *level),
                stacking_order: levels.get(&layout).map(|(order, _)| *order),
                // Keep all ancestors so later visibility/clip changes are
                // evaluated by execution, rather than cached into the plan.
                clip: Some(layout),
            });
        } else {
            nodes.push(PaintTree { before: vec![drawables[start].clone()], children: Vec::new(), after: Vec::new(), positioned: false, z_index: None, opacity: 1.0, stacking_order: None, clip: None });
        }
    }
    nodes.reverse();
    nodes
}
