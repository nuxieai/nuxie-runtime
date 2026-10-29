//! The three context-ownership tests added to
//! `tests/unit_tests/runtime/data_bind_container_test.cpp` at upstream
//! 6951a4b3d1344bf965c9975a8670730542fbada2.

use nuxie_runtime::source::{
    artboard::Artboard,
    core::{CoreArena, CoreHandle},
    data_bind::{
        converters::{
            data_converter::{DataConverter, bind_converter_context},
            data_converter_formula::DataConverterFormula,
            data_converter_group::DataConverterGroup,
            data_converter_group_item::DataConverterGroupItem,
        },
        data_bind::{BINDINGS_TARGET, DataBind},
        data_bind_container::{DataBindContainer, DataBindContainerOwner},
        data_context::{DataContext, RuntimeDataContextHandle},
    },
    data_bind_flags::DataBindFlags,
    generated::data_bind::converters::data_converter_formula_base::DataConverterFormulaBase,
};

#[test]
fn container_owns_the_data_context_it_is_bound_to() {
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    assert_eq!(context.debugging_refcnt(), 1);

    {
        let container = DataBindContainer::default();
        assert!(container.data_bind_context().is_none());

        container.bind_data_binds_from_context(context.clone());
        assert!(container.data_bind_context().unwrap().ptr_eq(&context));
        // The getter's temporary handle is dropped before observing the count.
        assert_eq!(context.debugging_refcnt(), 2);

        container.unbind_data_binds();
        assert!(container.data_bind_context().is_none());
        assert_eq!(context.debugging_refcnt(), 1);

        container.bind_data_binds_from_context(context.clone());
        assert_eq!(context.debugging_refcnt(), 2);
    }

    assert_eq!(context.debugging_refcnt(), 1);
}

#[test]
fn clearing_the_container_context_clears_what_add_data_bind_reads() {
    // Rust backpointers identify a registered owner. Use the actual Artboard's
    // retained container so the upstream container-identity assertion exercises
    // the live owner, rather than installing an unrelated test container.
    let arena = CoreArena::default();
    let owner = arena.insert(Artboard::default());
    let container = owner.data_bind_container().unwrap();
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    container.bind_data_binds_from_context(context.clone());
    assert!(container.data_bind_context().unwrap().ptr_eq(&context));

    // Release every context reference before add_data_bind consults the member.
    container.set_data_bind_context(None);
    drop(context);

    assert!(container.data_bind_context().is_none());
    // ToTarget is the default flag, matching upstream makeBind(). Its test
    // subclass only overrides update callbacks, which this case never invokes.
    let bind = arena.insert(DataBind::default());
    assert_eq!(
        bind.with(|bind| bind.as_data_bind().unwrap().base.flags()),
        Some(u32::from(DataBindFlags::TO_TARGET.0))
    );
    container.add_data_bind(bind.clone());
    let bind_container = bind
        .with(|bind| bind.as_data_bind().unwrap().container())
        .flatten()
        .expect("add_data_bind registers the container backpointer");
    assert!(matches!(
        bind_container,
        DataBindContainerOwner::Authored(ref actual) if actual == &owner
    ));
    container.delete_data_binds();
}

#[test]
fn converter_unbind_releases_the_data_context() {
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    assert_eq!(context.debugging_refcnt(), 1);

    {
        let mut group = DataConverterGroup::default();
        group.bind_from_context(context.clone(), None);
        assert_eq!(context.debugging_refcnt(), 2);
        group.unbind();
        assert_eq!(context.debugging_refcnt(), 1);
    }

    {
        let mut formula = DataConverterFormula::default();
        formula.bind_from_context(context.clone(), None);
        assert_eq!(context.debugging_refcnt(), 2);
        formula.unbind();
        assert_eq!(context.debugging_refcnt(), 1);
    }

    assert_eq!(context.debugging_refcnt(), 1);
}

// Rust integration: a converter's own binding can observe that converter.
// Removing the observer must reenter the formula after its mutable borrow ends.
fn formula_with_observing_child(arena: &CoreArena) -> (CoreHandle, CoreHandle) {
    let formula = arena.insert(DataConverterFormula::default());
    let child = arena.insert(DataBind::new(
        u32::from(DataBindFlags::TO_SOURCE.0),
        u32::from(DataConverterFormulaBase::RANDOM_MODE_VALUE_PROPERTY_KEY),
        0,
    ));
    formula
        .data_bind_container()
        .unwrap()
        .add_data_bind(child.clone());
    child.with_mut(|child| {
        let child = child.as_data_bind_mut().unwrap();
        child.set_target(Some(formula.clone()));
        assert!(child.target_supports_push());
    });
    (formula, child)
}

fn assert_formula_observer(formula: &CoreHandle, child: &CoreHandle, observing: bool) {
    child.with_mut(|child| child.as_data_bind_mut().unwrap().set_dirt(0));
    formula.with_mut(|formula| {
        formula
            .core_mut()
            .notify_property_changed(DataConverterFormulaBase::RANDOM_MODE_VALUE_PROPERTY_KEY);
    });
    assert_eq!(
        child.with(|child| child.as_data_bind().unwrap().dirt() & BINDINGS_TARGET),
        Some(if observing { BINDINGS_TARGET } else { 0 })
    );
}

#[test]
fn formula_handle_unbind_detaches_its_child_observer_and_can_rebind() {
    let arena = CoreArena::default();
    let (formula, child) = formula_with_observing_child(&arena);
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    bind_converter_context(&formula, context.clone(), None);
    assert_eq!(context.debugging_refcnt(), 2);
    assert_formula_observer(&formula, &child, true);

    DataConverter::unbind_handle(&formula);
    assert_eq!(context.debugging_refcnt(), 1);
    assert_formula_observer(&formula, &child, false);

    child.with_mut(|child| {
        let child = child.as_data_bind_mut().unwrap();
        child.set_target(None);
        child.set_target(Some(formula.clone()));
    });
    bind_converter_context(&formula, context.clone(), None);
    assert_eq!(context.debugging_refcnt(), 2);
    assert_formula_observer(&formula, &child, true);
    DataConverter::unbind_handle(&formula);
    assert_eq!(context.debugging_refcnt(), 1);
    assert_formula_observer(&formula, &child, false);
}

#[test]
fn direct_group_unbind_detaches_formula_child_observer() {
    let arena = CoreArena::default();
    let (formula, child) = formula_with_observing_child(&arena);
    let mut item = DataConverterGroupItem::default();
    item.set_converter(Some(formula.clone()));
    let mut group = DataConverterGroup::default();
    group.add_item(arena.insert(item));
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    group.bind_from_context(context.clone(), None);
    assert_eq!(context.debugging_refcnt(), 3);
    assert_formula_observer(&formula, &child, true);

    group.unbind();
    assert_eq!(context.debugging_refcnt(), 1);
    assert_formula_observer(&formula, &child, false);
}

#[test]
fn direct_data_bind_unbind_detaches_formula_child_observer() {
    let arena = CoreArena::default();
    let (formula, child) = formula_with_observing_child(&arena);
    let parent = arena.insert(DataBind::default());
    parent.with_mut(|parent| {
        parent
            .as_data_bind_mut()
            .unwrap()
            .set_converter(Some(formula.clone()));
    });
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    bind_converter_context(&formula, context.clone(), Some(parent.clone()));
    assert_eq!(context.debugging_refcnt(), 2);
    assert_formula_observer(&formula, &child, true);

    parent.with_mut(|parent| parent.as_data_bind_mut().unwrap().unbind());
    assert_eq!(context.debugging_refcnt(), 1);
    assert_formula_observer(&formula, &child, false);
}

#[test]
fn direct_data_bind_unbind_tolerates_a_retired_converter() {
    let arena = CoreArena::default();
    let formula = arena.insert(DataConverterFormula::default());
    let parent = arena.insert(DataBind::default());
    parent.with_mut(|parent| {
        parent
            .as_data_bind_mut()
            .unwrap()
            .set_converter(Some(formula.clone()));
    });
    assert!(formula.remove_occurrence());
    assert!(!formula.is_alive());
    parent.with_mut(|parent| parent.as_data_bind_mut().unwrap().unbind());
}

#[test]
fn ordinary_converter_advances_its_owned_nested_interpolator() {
    use nuxie_runtime::source::{
        data_bind::converters::{
            data_converter_interpolator::DataConverterInterpolator,
            data_converter_rounder::DataConverterRounder,
        },
        viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
    };
    let arena = CoreArena::default();
    let outer = arena.insert(DataConverterRounder::default());
    let nested = arena.insert(DataConverterInterpolator::default());
    let source = arena.insert(ViewModelInstanceNumber::default());
    let child = arena.insert(DataBind::default());
    outer
        .data_bind_container()
        .unwrap()
        .add_data_bind(child.clone());
    child.with_mut(|child| {
        let child = child.as_data_bind_mut().unwrap();
        child.set_target(Some(outer.clone()));
        child.set_source(source);
        child.set_converter(Some(nested));
    });
    assert!(
        outer
            .with(|owner| owner.as_data_converter_capability().unwrap().may_advance())
            .unwrap()
    );
    // An uninitialized interpolator remains active even at elapsed zero. A
    // stateless outer converter must preserve that nested result.
    assert!(DataConverter::advance_handle(&outer, 0.0));
}

// Upstream 832825db: converter subscriptions belong to each converter's own
// occurrence. Exercise both the direct methods and the released capability
// handlers used by live binding, without treating one path as the other.
use nuxie_runtime::source::{
    data_bind::{
        converters::data_converter_operation_viewmodel::DataConverterOperationViewModel,
        data_bind::BINDINGS, data_values::data_value_number::DataValueNumber,
    },
    generated::{
        core_registry::CoreRegistry,
        viewmodel::{
            viewmodel_instance_base::ViewModelInstanceBase,
            viewmodel_instance_value_base::ViewModelInstanceValueBase,
        },
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_value::ValueDependentHandle,
    },
};

fn is_dependent(value: &CoreHandle, dependent: &CoreHandle) -> bool {
    value
        .with(|value| value.as_view_model_instance_value().unwrap().dependents())
        .unwrap()
        .iter()
        .any(|entry| matches!(entry, ValueDependentHandle::Core(actual) if actual == dependent))
}
fn dependent_count(value: &CoreHandle) -> usize {
    value
        .with(|value| {
            value
                .as_view_model_instance_value()
                .unwrap()
                .dependents()
                .len()
        })
        .unwrap()
}
fn number_context(arena: &CoreArena) -> (RuntimeDataContextHandle, CoreHandle) {
    let instance = arena.insert(ViewModelInstance::default());
    assert!(CoreRegistry::set_uint_handle(
        &instance,
        ViewModelInstanceBase::VIEW_MODEL_ID_PROPERTY_KEY.into(),
        1
    ));
    let number = arena.insert(ViewModelInstanceNumber::default());
    assert!(CoreRegistry::set_uint_handle(
        &number,
        ViewModelInstanceValueBase::VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY.into(),
        2
    ));
    instance
        .with_downcast_mut::<ViewModelInstance, _>(|instance| instance.add_value(number.clone()))
        .unwrap();
    (
        RuntimeDataContextHandle::new(DataContext::new(Some(instance))),
        number,
    )
}
fn operation(arena: &CoreArena) -> CoreHandle {
    let mut converter = DataConverterOperationViewModel::default();
    converter.decode_source_path_ids(&[1, 2]);
    arena.insert(converter)
}
fn bind_test_converter(
    converter: &CoreHandle,
    context: RuntimeDataContextHandle,
    bind: &CoreHandle,
    released: bool,
) {
    if released {
        bind_converter_context(converter, context, Some(bind.clone()));
    } else {
        converter
            .with_mut(|converter| {
                if let Some(formula) = converter
                    .as_any_mut()
                    .downcast_mut::<DataConverterFormula>()
                {
                    formula.bind_from_context(context, Some(bind.clone()));
                } else {
                    converter
                        .as_any_mut()
                        .downcast_mut::<DataConverterOperationViewModel>()
                        .unwrap()
                        .bind_from_context(context, Some(bind.clone()));
                }
            })
            .unwrap();
    }
}
fn unbind_test_converter(converter: &CoreHandle, released: bool) {
    if released {
        DataConverter::unbind_handle(converter);
    } else {
        converter
            .with_mut(|converter| {
                converter
                    .as_data_converter_capability_mut()
                    .unwrap()
                    .unbind()
            })
            .unwrap();
    }
}
fn converted_number(converter: &CoreHandle, input: f32) -> f32 {
    converter
        .with_downcast_mut::<DataConverterOperationViewModel, _>(|converter| {
            converter
                .convert(&DataValueNumber::new(input))
                .as_any()
                .downcast_ref::<DataValueNumber>()
                .unwrap()
                .value()
        })
        .unwrap()
}

#[test]
fn formula_rebind_leaves_the_previous_source() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let context = RuntimeDataContextHandle::new(DataContext::new(None));
        let first = arena.insert(ViewModelInstanceEnum::default());
        let second = arena.insert(ViewModelInstanceEnum::default());
        let bind = arena.insert(DataBind::default());
        let formula = arena.insert(DataConverterFormula::default());
        bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_source(first.clone()));
        bind_test_converter(&formula, context.clone(), &bind, released);
        assert!(is_dependent(&first, &formula));
        bind.with_mut(|bind| {
            let bind = bind.as_data_bind_mut().unwrap();
            bind.clear_source();
            bind.set_source(second.clone());
        });
        bind_test_converter(&formula, context, &bind, released);
        assert!(!is_dependent(&first, &formula));
        assert!(is_dependent(&second, &formula));
        unbind_test_converter(&formula, released);
        assert!(!is_dependent(&second, &formula));
        assert!(bind.remove_occurrence());
    }
}

#[test]
fn operation_view_model_unbind_unsubscribes_its_own_identity() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let (context, first) = number_context(&arena);
        let bind = arena.insert(DataBind::default());
        let converter = operation(&arena);
        bind_test_converter(&converter, context.clone(), &bind, released);
        bind_test_converter(&converter, context, &bind, released);
        assert_eq!(dependent_count(&first), 1);
        assert!(is_dependent(&first, &converter));
        unbind_test_converter(&converter, released);
        assert!(!is_dependent(&first, &converter));
        assert!(bind.remove_occurrence());
    }
}

#[test]
fn operation_view_model_rebind_leaves_its_previous_operand() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let (first_context, first) = number_context(&arena);
        let (second_context, second) = number_context(&arena);
        let bind = arena.insert(DataBind::default());
        let converter = operation(&arena);
        bind_test_converter(&converter, first_context, &bind, released);
        bind_test_converter(&converter, second_context, &bind, released);
        assert!(!is_dependent(&first, &converter));
        assert!(is_dependent(&second, &converter));
        unbind_test_converter(&converter, released);
        assert!(bind.remove_occurrence());
    }
}

#[test]
fn operation_view_model_destroyed_while_bound_unsubscribes() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let (context, first) = number_context(&arena);
        let bind = arena.insert(DataBind::default());
        let converter = operation(&arena);
        bind_test_converter(&converter, context, &bind, released);
        assert!(is_dependent(&first, &converter));
        // Keep the old identity to test actual removal, not liveness filtering.
        assert!(converter.remove_occurrence());
        assert!(!is_dependent(&first, &converter));
        assert_eq!(dependent_count(&first), 0);
        assert!(bind.remove_occurrence());
    }
}

#[test]
fn operation_view_model_operand_changes_rerun_the_bind() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let (context, first) = number_context(&arena);
        let bind = arena.insert(DataBind::default());
        let converter = operation(&arena);
        bind_test_converter(&converter, context, &bind, released);
        bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_dirt(0));
        assert!(ViewModelInstanceNumber::set_value_handle(&first, 5.0));
        assert_eq!(
            bind.with(|bind| bind.as_data_bind().unwrap().dirt()),
            Some(BINDINGS)
        );
        unbind_test_converter(&converter, released);
        assert!(bind.remove_occurrence());
    }
}

#[test]
fn operation_view_model_rebind_keeps_subscriptions_it_shares() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let (first_context, first) = number_context(&arena);
        let (second_context, second) = number_context(&arena);
        let bind = arena.insert(DataBind::default());
        bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_source(first.clone()));
        let moved = operation(&arena);
        let kept = operation(&arena);
        bind_test_converter(&moved, first_context.clone(), &bind, released);
        bind_test_converter(&kept, first_context, &bind, released);
        bind_test_converter(&moved, second_context, &bind, released);
        assert!(is_dependent(&first, &bind));
        assert!(is_dependent(&first, &kept));
        assert!(!is_dependent(&first, &moved));
        assert!(is_dependent(&second, &moved));
        unbind_test_converter(&moved, released);
        unbind_test_converter(&kept, released);
        bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().clear_source());
        assert!(bind.remove_occurrence());
    }
}

#[test]
fn operation_rebinding_to_an_unresolved_operand_reads_zero() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let (context, first) = number_context(&arena);
        let missing = RuntimeDataContextHandle::new(DataContext::new(None));
        let bind = arena.insert(DataBind::default());
        let converter = operation(&arena);
        assert!(ViewModelInstanceNumber::set_value_handle(&first, 5.0));
        bind_test_converter(&converter, context, &bind, released);
        assert_eq!(converted_number(&converter, 2.0), 7.0);
        bind_test_converter(&converter, missing, &bind, released);
        assert!(!is_dependent(&first, &converter));
        assert_eq!(converted_number(&converter, 2.0), 2.0);
        unbind_test_converter(&converter, released);
    }
}

#[test]
fn formula_rebinding_without_a_bind_source_detaches_the_previous_source() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let context = RuntimeDataContextHandle::new(DataContext::new(None));
        let first = arena.insert(ViewModelInstanceEnum::default());
        let bind = arena.insert(DataBind::default());
        let formula = arena.insert(DataConverterFormula::default());
        bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_source(first.clone()));
        bind_test_converter(&formula, context.clone(), &bind, released);
        bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().clear_source());
        bind_test_converter(&formula, context, &bind, released);
        assert!(!is_dependent(&first, &formula));
        unbind_test_converter(&formula, released);
    }
}

#[test]
fn operation_same_operand_rebind_updates_its_parent_without_duplicate_subscription() {
    for released in [false, true] {
        let arena = CoreArena::default();
        let (context, number) = number_context(&arena);
        let first_bind = arena.insert(DataBind::default());
        let second_bind = arena.insert(DataBind::default());
        let converter = operation(&arena);
        bind_test_converter(&converter, context.clone(), &first_bind, released);
        bind_test_converter(&converter, context, &second_bind, released);
        assert_eq!(dependent_count(&number), 1);
        for bind in [&first_bind, &second_bind] {
            bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_dirt(0));
        }
        assert!(ViewModelInstanceNumber::set_value_handle(&number, 5.0));
        assert_eq!(
            first_bind.with(|bind| bind.as_data_bind().unwrap().dirt()),
            Some(0)
        );
        assert_eq!(
            second_bind.with(|bind| bind.as_data_bind().unwrap().dirt()),
            Some(BINDINGS)
        );
        unbind_test_converter(&converter, released);
    }
}

thread_local! {
    static REENTER_OPERATION: std::cell::RefCell<Option<CoreHandle>> = const { std::cell::RefCell::new(None) };
}
fn unbind_operation_during_parent_notification() {
    let converter = REENTER_OPERATION
        .with(|slot| slot.borrow_mut().take())
        .unwrap();
    DataConverter::unbind_handle(&converter);
}
#[test]
fn operation_dirt_forwarding_releases_converter_before_parent_reentry() {
    let arena = CoreArena::default();
    let (context, number) = number_context(&arena);
    let bind = arena.insert(DataBind::default());
    let converter = operation(&arena);
    bind_test_converter(&converter, context, &bind, true);
    bind.with_mut(|bind| {
        let bind = bind.as_data_bind_mut().unwrap();
        bind.set_dirt(0);
        bind.set_changed_callback(unbind_operation_during_parent_notification);
    });
    REENTER_OPERATION.with(|slot| *slot.borrow_mut() = Some(converter.clone()));
    assert!(ViewModelInstanceNumber::set_value_handle(&number, 5.0));
    assert!(REENTER_OPERATION.with(|slot| slot.borrow().is_none()));
    assert!(!is_dependent(&number, &converter));
    assert_eq!(
        bind.with(|bind| bind.as_data_bind().unwrap().dirt()),
        Some(BINDINGS)
    );
}
