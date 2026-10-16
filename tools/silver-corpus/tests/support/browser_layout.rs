//! UNIV-4844 CSS replacements for direct upstream SRIV assertions.
//! Every other case keeps its original exact comparison. A reviewed case
//! must retain this precise difference, including the original C++ value.
use silver_corpus::Difference;

pub fn reviewed_result(id: &str, result: Result<(), Difference>) -> Result<(), Difference> {
    let expected = match id {
        // CSS flex-shrink 1 lets the Fixed 409.5-wide nested box fit its 273-wide host instead of extending beyond it.
        "animated_clipping-layout" => {
            "frame 0, op 34 (addRawPath), field point: expected (409.5, 0), got (273, 0)"
        }
        // CSS automatic minimums sum seven 100-wide child artboards to a 700-wide content floor instead of the old 500 allocation.
        "artboard_list_overrides_vertical" => {
            "frame 0, op 73 (addRawPath), field point: expected (500, 0), got (700, 0)"
        }
        // CSS automatic minimums keep the lower content row at 740.94385 plus two 63-point padding edges, making the card 866.94385 instead of 850. Its 40-point corner leaves the first clip point at 826.94385 instead of 810.
        "bankcard" => {
            "frame 1, op 711 (addRawPath), field point: expected (810, 0), got (826.94385, 0)"
        }
        // CSS flex-shrink 1 lets the nested Fixed 500 box fit its 33.33 percent host, 499.95004, instead of keeping the full 500.
        "collapsing_elements" => {
            "frame 0, op 612 (addRawPath), field point: expected (500, -0.0 (0x80000000)), got (499.95004, -0.0 (0x80000000))"
        }
        // CSS flex-shrink 1 lets a Fixed child artboard fit the list row's height; the half-height origin moves from 29 to 12.5 as its box shrinks from 58 to 25.
        "component_list_child_origin" => {
            "frame 0, op 105 (transform), field ty: expected 29, got 12.5"
        }
        // CSS shrinking and content minimums change the chart cells' allocated widths; the first bar gradient endpoint moves from 276.66666 to 268.33334. Gradient interpolation is unchanged.
        "data_viz_demo" => {
            "frame 0, op 1289 (makeLinearGradient), field end: expected (276.66666, 16.325203), got (268.33334, 16.325203)"
        }
        // CSS automatic minimums propagate the nested Artboard Measure width 10000 through its containers: panel width 1008 becomes 10174 and the padded ancestor 1048 becomes 10214.
        "db_health_tracker" => {
            "frame 0, op 4764 (addRawPath), field point: expected (968, 0), got (10134, 0)"
        }
        // CSS automatic minimums preserve the Main artboard width 500 plus the GlobalName list width 480, or 980, instead of squeezing their parent to 945.
        "global_view_models_scripting_test" => {
            "frame 0, op 49 (addRawPath), field point: expected (945, 0), got (980, 0)"
        }
        // CSS min-height:auto keeps the nested 572-high content above the 208.60547 column offer; flex-shrink 1 reduces the leading Fixed 100 box to zero and moves the following item from y 100 to zero.
        "hittest_ab1_grand_parent" => "frame 0, op 149 (transform), field ty: expected 100, got 0",
        // CSS min-width:auto floors parent-clipper at the content contributions 238.65234 plus nested width 500, or 738.65234, instead of squeezing it to 572.
        "hittest_ab1_parent" => {
            "frame 0, op 129 (addRawPath), field point: expected (572, 0), got (738.65234, 0)"
        }
        // CSS automatic minimums keep the nested list row at its 61 content height rather than Yoga's 59.791016 allocation.
        "number_to_list_nested_children" => {
            "frame 0, op 190 (addRawPath), field point: expected (250, 59.791016), got (250, 61)"
        }
        // CSS automatic minimums preserve three 62-wide nested child artboards, or 186, instead of squeezing their background to 180.
        "reset_phase_multi_main" => {
            "frame 0, op 57 (addRawPath), field point: expected (180, 0), got (186, 0)"
        }
        // CSS automatic minimums preserve the 1500 content width rather than the 500 viewport allocation; inside it, Fixed bases 500 and 1500 shrink proportionally to 375 and 1125. Scroll dispatch is unchanged.
        "scroll_threshold-all-scroll" => {
            "frame 0, op 62 (addRawPath), field point: expected (500, 0), got (1500, 0)"
        }
        // CSS automatic minimums preserve the 1700 content width from the 200 and 1500 children rather than the 500 viewport allocation. Scroll dispatch is unchanged.
        "scroll_threshold-horizontal-scroll" => {
            "frame 0, op 59 (addRawPath), field point: expected (500, 0), got (1700, 0)"
        }
        // CSS shrinking and content minimums redistribute the nested Summary row's space and its fitted image transform. The first image scale becomes 0.218994141 in the ordinary lane and 0.429492176 in the scripted lane instead of 0.674160182; image fitting and drawing code are unchanged.
        "superbowl" => "frame 0, op 2964 (transform), field xx: expected 0.6741602, got 0.42949218",
        // CSS automatic minimums preserve the unchanged text-input measurer's unbounded 492.99805 contribution instead of squeezing the box to 200 in Silver or 340 in Golden. Text input still needs its own true minimum contract.
        "text_input" => {
            "frame 0, op 20 (addRawPath), field point: expected (200, 0), got (492.99805, 0)"
        }
        // CSS lets the small Hug item shrink from 17.300781 to 4.107422 in the 87-wide clock row while its 49.89258 sibling keeps its content floor; the following origin moves from 35.7041 to 29.107422.
        "text_layout_7_3" => {
            "frame 0, op 137 (transform), field tx: expected 35.7041, got 29.107422"
        }
        // CSS lets the small Hug item shrink from 17.300781 to 4.107422 in the 87-wide clock row while its 49.89258 sibling keeps its content floor; the following origin moves from 35.7041 to 29.107422.
        "text_layout_pre_7_3" => {
            "frame 0, op 137 (transform), field tx: expected 35.7041, got 29.107422"
        }
        _ => return result,
    };
    let difference = result.expect_err("reviewed CSS layout difference must remain present");
    assert_eq!(
        difference.to_string(),
        expected,
        "{id}: CSS layout pin moved"
    );
    Ok(())
}
