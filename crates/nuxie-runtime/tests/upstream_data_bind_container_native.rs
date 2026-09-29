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
