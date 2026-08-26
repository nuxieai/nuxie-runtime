/// Direct update for the embedded `TextVariationHelper` dependency owner.
///
/// C++ `TextVariationHelper::update` calls the retained TextStyle's accepted
/// `updateVariableFont` owner for every scheduled dirt mask and publishes no
/// fresh dirt. The helper-to-Text dependency has already propagated the source
/// dirt before this update node runs (`text_variation_helper.cpp:7-17`).
pub(crate) fn update_text_variation_helper(
    instance: &mut ArtboardInstance,
    style_local: usize,
    _text: crate::components::ComponentHandle,
    _dirt: crate::components::ComponentDirt,
) {
    let replacement = instance.runtime_file().and_then(|runtime| {
        instance.runtime_graph().and_then(|graph| {
            StaticTextStyle::from_graph_with_occurrence(runtime, graph, Some(instance), style_local)
                .ok()?
                .variable_font_replacement(runtime, instance)
        })
    });
    let Some(replacement) = replacement else {
        // Pinned updateVariableFont returns without touching an existing
        // variable font when the replacement base font is not ready.
        return;
    };
    if let Some(style) = instance
        .component_mut(style_local)
        .and_then(|component| component.concrete.text_style.as_mut())
    {
        style.update_variable_font(replacement);
    }
}
