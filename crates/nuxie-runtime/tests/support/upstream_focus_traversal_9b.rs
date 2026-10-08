//! The two fixture-backed focus cases from upstream 9b9cd7b1.
use super::*;
use nuxie_runtime::source::viewmodel::runtime::viewmodel_instance_runtime::RuntimeViewModelInstanceHandle;

fn default_instance(fixture: &BindingSilver) -> RuntimeViewModelInstanceHandle {
    let model = fixture
        .file
        .with_file(|file| file.default_artboard_view_model(Some(fixture.artboard.core_handle())))
        .expect("default artboard view model");
    let instance = model.create_default_instance();
    fixture.machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(instance.instance());
    });
    instance
}

fn rows(instance: &RuntimeViewModelInstanceHandle) -> Vec<RuntimeViewModelInstanceHandle> {
    let list = instance
        .property_list("layoutChildren")
        .expect("layoutChildren");
    assert_eq!(list.size(), 3);
    (0..3)
        .map(|i| list.instance_at(i).expect("row instance"))
        .collect()
}

fn set_leaf(
    instance: &RuntimeViewModelInstanceHandle,
    child: &str,
    focusable: bool,
    traversable: bool,
) {
    instance
        .property_boolean(&format!("{child}/focusable"))
        .expect("focusable")
        .set_value(focusable);
    instance
        .property_boolean(&format!("{child}/traversable"))
        .expect("traversable")
        .set_value(traversable);
}

fn assert_primary(manager: &RuntimeFocusManagerHandle, expected: &FocusNodeRef) {
    assert!(binding_focus_ptr_eq(
        &binding_primary(manager),
        &Some(expected.clone())
    ));
}

fn collect_order(manager: &RuntimeFocusManagerHandle, forward: bool) -> Vec<FocusNodeRef> {
    manager.with_focus_manager_mut(|manager| {
        manager.clear_focus();
        let mut order = Vec::new();
        for _ in 0..32 {
            let moved = if forward {
                manager.focus_next()
            } else {
                manager.focus_previous()
            };
            if !moved {
                break;
            }
            let Some(node) = manager.primary_focus() else {
                break;
            };
            order.push(node);
        }
        order
    })
}

fn assert_order(manager: &RuntimeFocusManagerHandle, expected: &[&FocusNodeRef], reverse: bool) {
    let actual = collect_order(manager, !reverse);
    let expected: Vec<_> = if reverse {
        expected.iter().rev().copied().collect()
    } else {
        expected.to_vec()
    };
    assert_eq!(actual.len(), expected.len(), "focus order length");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert!(Rc::ptr_eq(actual, expected), "focus order at {index}");
    }
}

#[test]
fn a_click_requests_focus_only_while_the_child_is_focusable() {
    let fixture = BindingSilver::new("focus_traversal_test.riv");
    let instance = default_instance(&fixture);
    let row_instances = rows(&instance);
    set_leaf(&instance, "node/child1", true, true);
    set_leaf(&instance, "node/child2", true, true);
    for row in &row_instances {
        set_leaf(row, "child1", true, true);
        set_leaf(row, "child2", true, true);
    }
    fixture.advance(0.016);
    fixture.advance(0.016);
    let manager = fixture
        .machine
        .with_instance(|machine| machine.focus_manager())
        .expect("focus manager");
    let roots = manager.with_focus_manager(|manager| manager.root_nodes().to_vec());
    assert_eq!(roots.len(), 2);
    let nested_parent = roots[0].borrow().children()[0].clone();
    assert_eq!(nested_parent.borrow().children().len(), 2);
    let mut children = nested_parent.borrow().children().to_vec();
    let list_scope = roots[1].borrow().children()[0].clone();
    assert_eq!(list_scope.borrow().children().len(), 3);
    for row in list_scope.borrow().children() {
        let parent = row.borrow().children()[0].clone();
        assert_eq!(parent.borrow().children().len(), 2);
        children.extend(parent.borrow().children().iter().cloned());
    }
    assert_eq!(children.len(), 8);
    let mut renderer = fixture.silver.borrow().make_renderer();
    let mut click_center_of = |node: &FocusNodeRef| {
        let focusable = node.borrow().focusable().expect("focusable backing");
        let bounds = focusable
            .borrow()
            .world_bounds()
            .expect("live world bounds");
        let center = Vec2D::new(
            (bounds.min_x + bounds.max_x) * 0.5,
            (bounds.min_y + bounds.max_y) * 0.5,
        );
        fixture.machine.with_instance_mut(|machine| {
            machine.pointer_down(
                center,
                0,
                nuxie_runtime::source::pointer_button::PointerButton::Primary,
            );
            machine.pointer_up(
                center,
                0,
                nuxie_runtime::source::pointer_button::PointerButton::Primary,
            );
        });
        fixture.advance(0.016);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
    };
    for child in &children {
        manager.with_focus_manager_mut(FocusManager::clear_focus);
        click_center_of(child);
        assert_primary(&manager, child);
    }
    let nested_first = &children[0];
    let nested_second = &children[1];
    set_leaf(&instance, "node/child2", false, true);
    fixture.advance(0.016);
    assert!(!nested_first.borrow().can_focus());
    manager.with_focus_manager_mut(|manager| {
        manager.clear_focus();
        manager.set_focus(nested_second.clone());
    });
    assert_primary(&manager, nested_second);
    click_center_of(nested_first);
    assert_primary(&manager, nested_second);
    set_leaf(&instance, "node/child2", true, true);
    fixture.advance(0.016);
    assert!(nested_first.borrow().can_focus());
    click_center_of(nested_first);
    assert_primary(&manager, nested_first);

    let middle_row_second = &children[5];
    let last_row_second = &children[7];
    set_leaf(&row_instances[1], "child1", false, true);
    fixture.advance(0.016);
    assert!(!middle_row_second.borrow().can_focus());
    assert!(last_row_second.borrow().can_focus());
    manager.with_focus_manager_mut(|manager| {
        manager.clear_focus();
        manager.set_focus(last_row_second.clone());
    });
    click_center_of(middle_row_second);
    assert_primary(&manager, last_row_second);
    click_center_of(&children[3]);
    assert_primary(&manager, &children[3]);
    set_leaf(&row_instances[1], "child1", true, true);
    fixture.advance(0.016);
    click_center_of(middle_row_second);
    assert_primary(&manager, middle_row_second);

    set_leaf(&instance, "node/child1", true, false);
    fixture.advance(0.016);
    assert!(nested_second.borrow().can_focus());
    assert!(!nested_second.borrow().can_traverse());
    manager.with_focus_manager_mut(FocusManager::clear_focus);
    click_center_of(nested_second);
    assert_primary(&manager, nested_second);
    set_leaf(&instance, "node/child1", false, false);
    fixture.advance(0.016);
    manager.with_focus_manager_mut(|manager| {
        manager.clear_focus();
        manager.set_focus(nested_first.clone());
    });
    click_center_of(nested_second);
    assert_primary(&manager, nested_first);
    drop(click_center_of);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("focus_traversal_click_to_focus");
}

#[test]
fn data_bound_focus_flags_drive_traversal_through_a_nested_artboard_and_a_list() {
    let fixture = BindingSilver::new("focus_traversal_test.riv");
    let instance = default_instance(&fixture);
    fixture.advance(0.016);
    let manager = fixture
        .machine
        .with_instance(|machine| machine.focus_manager())
        .expect("focus manager");
    let roots = manager.with_focus_manager(|manager| manager.root_nodes().to_vec());
    assert_eq!(roots.len(), 2);
    let nested_host = &roots[0];
    let list_host = &roots[1];
    assert!(nested_host.borrow().can_focus());
    assert!(!nested_host.borrow().can_traverse());
    assert!(list_host.borrow().can_focus());
    assert!(!list_host.borrow().can_traverse());
    assert_eq!(nested_host.borrow().children().len(), 1);
    let nested_parent = nested_host.borrow().children()[0].clone();
    assert!(nested_parent.borrow().can_focus());
    assert!(nested_parent.borrow().can_traverse());
    assert_eq!(nested_parent.borrow().children().len(), 2);
    let nested_leaf_a = nested_parent.borrow().children()[0].clone();
    let nested_leaf_b = nested_parent.borrow().children()[1].clone();
    assert_eq!(list_host.borrow().children().len(), 1);
    let list_scope = list_host.borrow().children()[0].clone();
    assert_eq!(list_scope.borrow().children().len(), 3);
    let mut row_parents = Vec::new();
    let mut row_leaf_a = Vec::new();
    let mut row_leaf_b = Vec::new();
    for row in list_scope.borrow().children() {
        assert_eq!(row.borrow().children().len(), 1);
        let parent = row.borrow().children()[0].clone();
        assert_eq!(parent.borrow().children().len(), 2);
        row_leaf_a.push(parent.borrow().children()[0].clone());
        row_leaf_b.push(parent.borrow().children()[1].clone());
        row_parents.push(parent);
    }
    let a_focusable = instance
        .property_boolean("node/child2/focusable")
        .expect("boolean");
    let a_traversable = instance
        .property_boolean("node/child2/traversable")
        .expect("boolean");
    let b_focusable = instance
        .property_boolean("node/child1/focusable")
        .expect("boolean");
    let b_traversable = instance
        .property_boolean("node/child1/traversable")
        .expect("boolean");
    let row_instances = rows(&instance);
    let mut renderer = fixture.silver.borrow().make_renderer();
    let mut settle_and_draw = || {
        fixture.advance(0.016);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
    };
    let parents = [
        &nested_parent,
        &row_parents[0],
        &row_parents[1],
        &row_parents[2],
    ];
    settle_and_draw();
    assert_order(&manager, &parents, false);
    assert_order(&manager, &parents, true);

    b_focusable.set_value(true);
    settle_and_draw();
    assert!(nested_leaf_b.borrow().can_focus());
    assert!(!nested_leaf_b.borrow().can_traverse());
    assert_order(&manager, &parents, false);
    manager.with_focus_manager_mut(|manager| manager.set_focus(nested_leaf_b.clone()));
    assert_primary(&manager, &nested_leaf_b);

    b_focusable.set_value(false);
    b_traversable.set_value(true);
    settle_and_draw();
    assert!(!nested_leaf_b.borrow().can_focus());
    assert!(nested_leaf_b.borrow().can_traverse());
    assert_order(&manager, &parents, false);
    manager.with_focus_manager_mut(|manager| {
        manager.clear_focus();
        manager.set_focus(nested_leaf_b.clone());
    });
    assert!(binding_primary(&manager).is_none());

    b_focusable.set_value(true);
    settle_and_draw();
    assert_order(
        &manager,
        &[
            &nested_parent,
            &nested_leaf_b,
            &row_parents[0],
            &row_parents[1],
            &row_parents[2],
        ],
        false,
    );

    a_focusable.set_value(true);
    a_traversable.set_value(true);
    settle_and_draw();
    let nested_and_parents = [
        &nested_parent,
        &nested_leaf_a,
        &nested_leaf_b,
        &row_parents[0],
        &row_parents[1],
        &row_parents[2],
    ];
    assert_order(&manager, &nested_and_parents, false);
    assert_order(&manager, &nested_and_parents, true);

    set_leaf(&row_instances[1], "child2", true, true);
    set_leaf(&row_instances[1], "child1", true, true);
    settle_and_draw();
    assert_order(
        &manager,
        &[
            &nested_parent,
            &nested_leaf_a,
            &nested_leaf_b,
            &row_parents[0],
            &row_parents[1],
            &row_leaf_a[1],
            &row_leaf_b[1],
            &row_parents[2],
        ],
        false,
    );

    for row in &row_instances {
        set_leaf(row, "child1", true, true);
        set_leaf(row, "child2", true, true);
    }
    settle_and_draw();
    let all = [
        &nested_parent,
        &nested_leaf_a,
        &nested_leaf_b,
        &row_parents[0],
        &row_leaf_a[0],
        &row_leaf_b[0],
        &row_parents[1],
        &row_leaf_a[1],
        &row_leaf_b[1],
        &row_parents[2],
        &row_leaf_a[2],
        &row_leaf_b[2],
    ];
    assert_order(&manager, &all, false);
    assert_order(&manager, &all, true);

    set_leaf(&row_instances[0], "child1", false, false);
    set_leaf(&row_instances[0], "child2", false, false);
    settle_and_draw();
    assert_order(
        &manager,
        &[
            &nested_parent,
            &nested_leaf_a,
            &nested_leaf_b,
            &row_parents[0],
            &row_parents[1],
            &row_leaf_a[1],
            &row_leaf_b[1],
            &row_parents[2],
            &row_leaf_a[2],
            &row_leaf_b[2],
        ],
        false,
    );
    drop(settle_and_draw);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("focus_traversal_data_bound");
}
