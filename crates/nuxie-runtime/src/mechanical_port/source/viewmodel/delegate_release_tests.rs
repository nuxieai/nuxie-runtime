//! A value's delegates never run while that value's Core slot is borrowed.
//!
//! The borrowed setters (`set_value(&mut self)`, `apply_value`) reach
//! `ViewModelInstanceValue::on_value_changed` from inside `with_mut`. A
//! delegate that reads its own value (a script listener reading
//! `property.value`, through `CoreHandle::with`) used to hit
//! `RefCell already mutably borrowed` there.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    core::{CoreArena, CoreHandle},
    data_bind::data_values::data_value_integer::DataValueInteger,
    viewmodel::{
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
        viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob,
        viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
        viewmodel_instance_asset_image::ViewModelInstanceAssetImage,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
        viewmodel_instance_value::{
            ViewModelInstanceValueDelegate, ViewModelInstanceValueDelegateHandle,
        },
    },
};

/// Reads its own value back through the shared Core accessor, as the
/// scripted `value` getter does.
struct ReadingDelegate {
    value: CoreHandle,
    calls: Rc<Cell<u32>>,
    readable: Rc<Cell<u32>>,
}

impl ViewModelInstanceValueDelegate for ReadingDelegate {
    fn value_changed(&mut self) {
        self.calls.set(self.calls.get() + 1);
        if self
            .value
            .with(|value| value.as_view_model_instance_value().is_some())
            == Some(true)
        {
            self.readable.set(self.readable.get() + 1);
        }
    }
}

struct Observed {
    _delegate: ViewModelInstanceValueDelegateHandle,
    calls: Rc<Cell<u32>>,
    readable: Rc<Cell<u32>>,
}

fn observe(value: &CoreHandle) -> Observed {
    let calls = Rc::new(Cell::new(0));
    let readable = Rc::new(Cell::new(0));
    let delegate: ViewModelInstanceValueDelegateHandle = Rc::new(RefCell::new(ReadingDelegate {
        value: value.clone(),
        calls: calls.clone(),
        readable: readable.clone(),
    }));
    value
        .with_mut(|value| {
            value
                .as_view_model_instance_value_mut()
                .map(|value| value.add_delegate(&delegate))
        })
        .flatten()
        .unwrap();
    Observed {
        _delegate: delegate,
        calls,
        readable,
    }
}

fn assert_notified_once(name: &str, observed: &Observed) {
    assert_eq!(observed.calls.get(), 1, "{name}: delegate call count");
    assert_eq!(
        observed.readable.get(),
        1,
        "{name}: the delegate could read its own value"
    );
}

/// Every borrowed setter, written the way `apply_source_value` writes a
/// non-number source (`source.with_mut(|source| source.apply_value(..))`).
fn write_each_type(arena: &CoreArena, write: &dyn Fn(&CoreHandle, &dyn Fn(&CoreHandle))) {
    let cases: Vec<(&str, CoreHandle, Box<dyn Fn(&CoreHandle)>)> = vec![
        (
            "string",
            arena.insert(ViewModelInstanceString::default()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceString, _>(|value| value.set_value("x"));
            }),
        ),
        (
            "color",
            arena.insert(ViewModelInstanceColor::default()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceColor, _>(|value| value.set_value(7));
            }),
        ),
        (
            "boolean",
            arena.insert(ViewModelInstanceBoolean::default()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceBoolean, _>(|value| {
                    value.set_value(true)
                });
            }),
        ),
        (
            "enum",
            arena.insert(ViewModelInstanceEnum::default()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceEnum, _>(|value| {
                    value.apply_value(&DataValueInteger::new(1))
                });
            }),
        ),
        (
            "symbol list index",
            arena.insert(ViewModelInstanceSymbolListIndex::default()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceSymbolListIndex, _>(|value| {
                    value.apply_value(&DataValueInteger::new(1))
                });
            }),
        ),
        (
            "artboard",
            arena.insert(ViewModelInstanceArtboard::default()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceArtboard, _>(|value| {
                    value.apply_value(&DataValueInteger::new(1))
                });
            }),
        ),
        (
            "image",
            arena.insert(ViewModelInstanceAssetImage::new()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceAssetImage, _>(|value| {
                    value.apply_value(&DataValueInteger::new(1))
                });
            }),
        ),
        (
            "font",
            arena.insert(ViewModelInstanceAssetFont::new()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceAssetFont, _>(|value| {
                    value.apply_value(&DataValueInteger::new(1))
                });
            }),
        ),
        (
            "blob",
            arena.insert(ViewModelInstanceAssetBlob::new()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceAssetBlob, _>(|value| {
                    value.apply_value(&DataValueInteger::new(1))
                });
            }),
        ),
        (
            "trigger",
            arena.insert(ViewModelInstanceTrigger::default()),
            Box::new(|value: &CoreHandle| {
                ViewModelInstanceTrigger::trigger_handle(value);
            }),
        ),
        (
            "number",
            arena.insert(ViewModelInstanceNumber::default()),
            Box::new(|value: &CoreHandle| {
                value.with_downcast_mut::<ViewModelInstanceNumber, _>(|value| value.set_value(2.0));
            }),
        ),
    ];
    for (name, value, set) in cases {
        let observed = observe(&value);
        write(&value, &*set);
        assert_notified_once(name, &observed);
    }
}

#[test]
fn a_borrowed_write_notifies_after_the_slot_is_released() {
    let arena = CoreArena::default();
    write_each_type(&arena, &|value, set| set(value));
}

#[test]
fn a_borrowed_write_in_a_host_mutation_notifies_after_the_slot_is_released() {
    // `native_binding.rs` `mutate` arms the host mutation buffer around the
    // data-bind to-source write; publication is not active there.
    let arena = CoreArena::default();
    write_each_type(&arena, &|value, set| {
        let notifications = crate::view_model_cell::RuntimeHostMutationNotifications::begin()
            .expect("no outer host mutation");
        set(value);
        notifications.commit();
    });
}

#[test]
fn a_borrowed_write_nested_in_another_borrow_notifies_once_every_borrow_ends() {
    let arena = CoreArena::default();
    let outer = arena.insert(ViewModelInstanceString::default());
    let value = arena.insert(ViewModelInstanceString::default());
    let observed = observe(&value);
    outer
        .with_mut(|_| {
            value.with_downcast_mut::<ViewModelInstanceString, _>(|value| value.set_value("x"));
            assert_eq!(observed.calls.get(), 0, "no delegate runs under a borrow");
        })
        .unwrap();
    assert_notified_once("nested", &observed);
}

#[test]
fn a_detached_value_notifies_at_once() {
    let mut value = ViewModelInstanceString::default();
    let calls = Rc::new(Cell::new(0));
    struct Counting(Rc<Cell<u32>>);
    impl ViewModelInstanceValueDelegate for Counting {
        fn value_changed(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let delegate: ViewModelInstanceValueDelegateHandle =
        Rc::new(RefCell::new(Counting(calls.clone())));
    value.base.add_delegate(&delegate);
    value.set_value("x");
    assert_eq!(calls.get(), 1);
}

#[test]
fn delegates_that_are_gone_leave_the_list_as_others_are_added() {
    let arena = CoreArena::default();
    let value = arena.insert(ViewModelInstanceString::default());
    let kept = observe(&value);
    for _ in 0..10_000 {
        // A watch registered and dropped again, as a script can do in a loop.
        let _gone = observe(&value);
    }
    let capacity = value
        .with(|value| {
            value
                .as_view_model_instance_value()
                .map(|value| value.delegates.capacity())
        })
        .flatten()
        .unwrap();
    assert!(capacity <= 4, "delegate list capacity {capacity}");
    value.with_downcast_mut::<ViewModelInstanceString, _>(|value| value.set_value("x"));
    assert_notified_once("kept", &kept);
}
