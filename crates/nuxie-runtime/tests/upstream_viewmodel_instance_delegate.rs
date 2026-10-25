//! Literal viewmodel_instance_delegate_test.cpp at upstream a9239045.
use nuxie_runtime::source::{
    core::{CoreArena, CoreHandle},
    viewmodel::{
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_value::{
            ViewModelInstanceValueDelegate, ViewModelInstanceValueDelegateHandle,
        },
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct CountingDelegate {
    calls: Rc<Cell<i32>>,
    on_change: Option<Box<dyn FnMut()>>,
}

impl ViewModelInstanceValueDelegate for CountingDelegate {
    fn value_changed(&mut self) {
        self.calls.set(self.calls.get() + 1);
        if let Some(on_change) = self.on_change.as_mut() {
            on_change();
        }
    }
}

fn delegate(
    calls: &Rc<Cell<i32>>,
    on_change: Option<Box<dyn FnMut()>>,
) -> ViewModelInstanceValueDelegateHandle {
    Rc::new(RefCell::new(CountingDelegate {
        calls: calls.clone(),
        on_change,
    }))
}

fn add(number: &CoreHandle, delegate: &ViewModelInstanceValueDelegateHandle) {
    number
        .with_mut(|number| {
            number
                .as_view_model_instance_value_mut()
                .unwrap()
                .add_delegate(delegate)
        })
        .unwrap();
}

#[test]
fn a_delegate_removed_during_a_change_is_not_called_for_it() {
    let arena = CoreArena::default();
    let number = arena.insert(ViewModelInstanceNumber::default());
    let first_calls = Rc::new(Cell::new(0));
    let second_calls = Rc::new(Cell::new(0));
    let second = delegate(&second_calls, None);
    let first = delegate(
        &first_calls,
        Some(Box::new({
            let number = number.clone();
            let second = second.clone();
            move || {
                number
                    .with_mut(|number| {
                        number
                            .as_view_model_instance_value_mut()
                            .unwrap()
                            .remove_delegate(&second)
                    })
                    .unwrap();
            }
        })),
    );
    add(&number, &first);
    add(&number, &second);
    ViewModelInstanceNumber::set_value_handle(&number, 1.0);
    assert_eq!(first_calls.get(), 1);
    assert_eq!(second_calls.get(), 0);
}

#[test]
fn a_change_made_by_a_delegate_leaves_the_delegates_being_called_alone() {
    let arena = CoreArena::default();
    let number = arena.insert(ViewModelInstanceNumber::default());
    let first_calls = Rc::new(Cell::new(0));
    let second_calls = Rc::new(Cell::new(0));
    let late_calls = Rc::new(Cell::new(0));
    let second = delegate(&second_calls, None);
    let late = delegate(&late_calls, None);
    let first = delegate(
        &first_calls,
        Some(Box::new({
            let number = number.clone();
            let first_calls = first_calls.clone();
            let second = second.clone();
            let late = late.clone();
            move || {
                if first_calls.get() == 1 {
                    number
                        .with_mut(|number| {
                            let number = number.as_view_model_instance_value_mut().unwrap();
                            number.remove_delegate(&second);
                            number.add_delegate(&late);
                        })
                        .unwrap();
                    ViewModelInstanceNumber::set_value_handle(&number, 2.0);
                }
            }
        })),
    );
    add(&number, &first);
    add(&number, &second);
    ViewModelInstanceNumber::set_value_handle(&number, 1.0);
    assert_eq!(second_calls.get(), 0);
    assert_eq!(late_calls.get(), 0);
    ViewModelInstanceNumber::set_value_handle(&number, 3.0);
    assert_eq!(first_calls.get(), 2);
    assert_eq!(late_calls.get(), 1);
}

// Rust retains delegate allocations through Weak handles, so placement-new
// address recycling is unavailable. Removing and readding the same identity
// exercises upstream's stricter rule: never join the change already in flight.
#[test]
fn a_delegate_readded_at_the_removed_identity_waits_for_the_next_change() {
    let arena = CoreArena::default();
    let number = arena.insert(ViewModelInstanceNumber::default());
    let calls = Rc::new(Cell::new(0));
    let removed = delegate(&calls, None);
    let added = Rc::new(Cell::new(false));
    let first = delegate(&Rc::new(Cell::new(0)), Some(Box::new({
        let number = number.clone();
        let removed = removed.clone();
        let added = added.clone();
        move || {
            if !added.replace(true) {
                number.with_mut(|number| {
                    let number = number.as_view_model_instance_value_mut().unwrap();
                    number.remove_delegate(&removed);
                    number.add_delegate(&removed);
                }).unwrap();
            }
        }
    })));
    add(&number, &first);
    add(&number, &removed);
    ViewModelInstanceNumber::set_value_handle(&number, 1.0);
    assert!(added.get());
    assert_eq!(calls.get(), 0);
    ViewModelInstanceNumber::set_value_handle(&number, 2.0);
    assert_eq!(calls.get(), 1);
    number.with_mut(|number| number.as_view_model_instance_value_mut().unwrap().remove_delegate(&removed));
}
