use nuxie_render_api::{
    RecordingFactory, SideChannelSemanticsBoundsUpdate, SideChannelSemanticsChildrenUpdate,
    SideChannelSemanticsDiff, SideChannelSemanticsNode,
};

#[test]
fn semantic_side_channel_serializes_the_complete_diff_in_oracle_order() {
    let node = SideChannelSemanticsNode {
        id: 7,
        role: 3,
        label: "Text \"label\"".to_owned(),
        value: "line\nvalue".to_owned(),
        hint: String::new(),
        state_flags: 5,
        trait_flags: 9,
        heading_level: 2,
        min_x: -1.25,
        min_y: 0.0,
        max_x: 10.5,
        max_y: 20.0,
        parent_id: -1,
        sibling_index: 4,
    };
    let diff = SideChannelSemanticsDiff {
        frame_number: 11,
        tree_version: 13,
        root_id: 7,
        removed: vec![2, 1],
        added: vec![node.clone()],
        moved: vec![node.clone()],
        children_updated: vec![SideChannelSemanticsChildrenUpdate {
            parent_id: -1,
            child_ids: vec![7, 8],
        }],
        updated_semantic: vec![node],
        updated_geometry: vec![SideChannelSemanticsBoundsUpdate {
            id: 7,
            min_x: -2.0,
            min_y: 1.0,
            max_x: 12.0,
            max_y: 21.5,
        }],
    };

    let mut factory = RecordingFactory::new();
    factory.add_semantics_diff(&diff);

    assert_eq!(
        factory.stream(),
        concat!(
            "rive-golden-stream-v1\n",
            "semantics frame=11 treeVersion=13 rootId=7\n",
            "semantics removed ids=[2,1]\n",
            "semantics added nodes=[{id=7,role=3,label=\"Text \\\"label\\\"\",value=\"line\\nvalue\",hint=\"\",stateFlags=5,traitFlags=9,headingLevel=2,bounds=(-1.25,0,10.5,20),parentId=-1,siblingIndex=4}]\n",
            "semantics moved nodes=[{id=7,role=3,label=\"Text \\\"label\\\"\",value=\"line\\nvalue\",hint=\"\",stateFlags=5,traitFlags=9,headingLevel=2,bounds=(-1.25,0,10.5,20),parentId=-1,siblingIndex=4}]\n",
            "semantics childrenUpdated entries=[{parentId=-1,childIds=[7,8]}]\n",
            "semantics updatedSemantic nodes=[{id=7,role=3,label=\"Text \\\"label\\\"\",value=\"line\\nvalue\",hint=\"\",stateFlags=5,traitFlags=9,headingLevel=2,bounds=(-1.25,0,10.5,20),parentId=-1,siblingIndex=4}]\n",
            "semantics updatedGeometry bounds=[{id=7,bounds=(-2,1,12,21.5)}]\n",
        )
    );
}

#[test]
fn semantic_side_channel_emits_all_vectors_for_an_empty_diff() {
    let mut factory = RecordingFactory::new();
    factory.add_semantics_diff(&SideChannelSemanticsDiff::default());

    assert_eq!(
        factory.stream(),
        concat!(
            "rive-golden-stream-v1\n",
            "semantics frame=0 treeVersion=0 rootId=0\n",
            "semantics removed ids=[]\n",
            "semantics added nodes=[]\n",
            "semantics moved nodes=[]\n",
            "semantics childrenUpdated entries=[]\n",
            "semantics updatedSemantic nodes=[]\n",
            "semantics updatedGeometry bounds=[]\n",
        )
    );
}

#[test]
fn semantic_input_outcomes_use_the_shared_stream_float_and_status_grammar() {
    let mut factory = RecordingFactory::new();
    factory.add_semantic_action(0.25, 9, "increase", true);
    factory.add_semantic_action(0.5, 99, "tap", false);
    factory.add_semantic_focus(0.75, 7, true);
    factory.add_semantic_focus(1.0, 8, false);

    assert_eq!(
        factory.stream(),
        concat!(
            "rive-golden-stream-v1\n",
            "semanticAction seconds=0.25 nodeId=9 action=increase outcome=dispatched\n",
            "semanticAction seconds=0.5 nodeId=99 action=tap outcome=missing\n",
            "semanticFocus seconds=0.75 nodeId=7 outcome=focused\n",
            "semanticFocus seconds=1 nodeId=8 outcome=rejected\n",
        )
    );
}
