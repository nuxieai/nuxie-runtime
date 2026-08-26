//! Exact ports of cases 1-15 from pinned `semantic_label_inference_test.cpp`.

use super::*;
use crate::SemanticRole;

fn node(id: u32, role: SemanticRole, label: &str) -> SemanticNodeHandle {
    let node = SemanticNodeHandle::new(id);
    {
        let mut owner = node.borrow_mut();
        owner.set_role(role as u32);
        owner.set_label(label);
    }
    node
}

fn drain(manager: &mut SemanticManager) -> SemanticsDiff {
    manager
        .drain_diff()
        .expect("pinned in-memory tree has no unresolved boundary dirt")
}

fn added(diff: &SemanticsDiff, id: u32) -> Option<&SemanticsDiffNode> {
    diff.added.iter().find(|node| node.id == id)
}

#[test]
fn wave_c17_001_button_derives_label_from_child_text() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let text = node(2, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), text);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Play");
    assert!(added(&diff, 2).is_none());
}

#[test]
fn wave_c17_002_button_with_explicit_label_ignores_children() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "Play media");
    let text = node(2, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), text);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Play media");
}

#[test]
fn wave_c17_003_multiple_text_children_concatenate_in_tree_order() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let text1 = node(2, SemanticRole::Text, "Play");
    let text2 = node(3, SemanticRole::Text, "now");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), text1);
    manager.add_child(Some(&button), text2);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Play now");
}

#[test]
fn wave_c17_004_button_derives_label_from_image_without_text() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let image = node(2, SemanticRole::Image, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), image);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Play");
}

#[test]
fn wave_c17_005_text_takes_priority_over_image_in_label_derivation() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let image = node(2, SemanticRole::Image, "Play icon");
    let text = node(3, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), image);
    manager.add_child(Some(&button), text);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Play");
}

#[test]
fn wave_c17_006_derived_label_trims_and_collapses_whitespace() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let text1 = node(2, SemanticRole::Text, "  Hello  ");
    let text2 = node(3, SemanticRole::Text, "  world  ");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), text1);
    manager.add_child(Some(&button), text2);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Hello world");
}

#[test]
fn wave_c17_007_absorbed_children_are_not_in_flat_output() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let icon = node(2, SemanticRole::None, "");
    let text = node(3, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), icon);
    manager.add_child(Some(&button), text);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Play");
    assert!(added(&diff, 2).is_none());
    assert!(added(&diff, 3).is_none());
}

#[test]
fn wave_c17_008_text_nested_in_group_contributes_to_button_label() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let group = node(2, SemanticRole::Group, "");
    let text = node(3, SemanticRole::Text, "Submit");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), group.clone());
    manager.add_child(Some(&group), text);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Submit");
    assert!(added(&diff, 2).is_none());
    assert!(added(&diff, 3).is_none());
}

#[test]
fn wave_c17_009_standalone_group_does_not_absorb_child_labels() {
    let mut manager = SemanticManager::new();
    let group = node(1, SemanticRole::Group, "Settings");
    let button = node(2, SemanticRole::Button, "WiFi");
    manager.add_child(None, group.clone());
    manager.add_child(Some(&group), button);
    let diff = drain(&mut manager);
    let group = added(&diff, 1).expect("group in added diff");
    assert_eq!(group.label, "Settings");
    let button = added(&diff, 2).expect("button in added diff");
    assert_eq!(button.label, "WiFi");
}

#[test]
fn wave_c17_010_text_field_does_not_derive_label_from_children() {
    let mut manager = SemanticManager::new();
    let text_field = node(1, SemanticRole::TextField, "");
    let text = node(2, SemanticRole::Text, "Search");
    manager.add_child(None, text_field.clone());
    manager.add_child(Some(&text_field), text);
    let diff = drain(&mut manager);
    let text_field = added(&diff, 1).expect("text field in added diff");
    assert_eq!(text_field.label, "");
    assert!(added(&diff, 2).is_some());
}

#[test]
fn wave_c17_011_explicit_label_on_button_overrides_child_text() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "Play media");
    let text = node(2, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), text);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "Play media");
    assert!(added(&diff, 2).is_some());
}

#[test]
fn wave_c17_012_all_interactive_roles_derive_labels_from_children() {
    for role in [
        SemanticRole::Button,
        SemanticRole::Link,
        SemanticRole::Checkbox,
        SemanticRole::SwitchControl,
        SemanticRole::Slider,
        SemanticRole::Tab,
        SemanticRole::ListItem,
    ] {
        let mut manager = SemanticManager::new();
        let parent = node(1, role, "");
        let text = node(2, SemanticRole::Text, "Label");
        manager.add_child(None, parent.clone());
        manager.add_child(Some(&parent), text);
        let diff = drain(&mut manager);
        let owner = added(&diff, 1).expect("interactive node in added diff");
        assert_eq!(owner.label, "Label", "role {role:?}");
    }
}

#[test]
fn wave_c17_013_interactive_node_without_children_has_empty_label() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    manager.add_child(None, button);
    let diff = drain(&mut manager);
    let button = added(&diff, 1).expect("button in added diff");
    assert_eq!(button.label, "");
}

#[test]
fn wave_c17_014_absorbed_child_content_change_rederives_parent_label() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let text = node(2, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), text.clone());
    let first = drain(&mut manager);
    let button = added(&first, 1).expect("button in first diff");
    assert_eq!(button.label, "Play");
    text.borrow_mut().set_label("Pause");
    manager.mark_node_dirty(2, SemanticDirt::CONTENT);
    let second = drain(&mut manager);
    let mut button_updated = false;
    for owner in &second.updated_semantic {
        if owner.id == 1 {
            assert_eq!(owner.label, "Pause");
            button_updated = true;
        }
    }
    assert!(button_updated);
}

#[test]
fn wave_c17_015_interactive_role_classification_is_exact() {
    for role in [
        SemanticRole::Button,
        SemanticRole::Link,
        SemanticRole::Checkbox,
        SemanticRole::SwitchControl,
        SemanticRole::Slider,
        SemanticRole::Tab,
        SemanticRole::ListItem,
    ] {
        assert!(is_interactive_role(role as u32), "role {role:?}");
    }
    for role in [
        SemanticRole::None,
        SemanticRole::TextField,
        SemanticRole::Image,
        SemanticRole::Text,
        SemanticRole::List,
        SemanticRole::TabList,
        SemanticRole::Group,
    ] {
        assert!(!is_interactive_role(role as u32), "role {role:?}");
    }
    assert!(is_interactive_role(SemanticRole::Button as u32));
    assert!(!is_interactive_role(SemanticRole::None as u32));
}
