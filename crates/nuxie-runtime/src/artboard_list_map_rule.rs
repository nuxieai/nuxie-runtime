// Direct owner for pinned C++ `src/artboard_list_map_rule.cpp`.

fn artboard_list_map_rule_for_view_model<'a>(
    rules: &'a [nuxie_graph::ComponentListMapRuleNode],
    view_model_index: usize,
) -> Option<&'a nuxie_graph::ComponentListMapRuleNode> {
    rules
        .iter()
        .find(|rule| rule.view_model_id == view_model_index as i64)
}
