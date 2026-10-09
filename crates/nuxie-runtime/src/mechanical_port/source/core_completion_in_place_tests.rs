//! Completion transport must preserve the existing released-setter boundary.
use super::*;
use crate::mechanical_port::source::{
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistry, CoreRegistryObject},
        node_base::NodeBase,
    },
    node::Node,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

const KEY: u16 = NodeBase::X_PROPERTY_KEY;
type WriteHook = Box<dyn FnMut(&Core, &mut PropertySetterCompletion)>;
thread_local! {
    static CALLBACK: RefCell<Option<Box<dyn FnOnce(&CoreHandle)>>> = const { RefCell::new(None) };
}
fn on_completion(owner: &CoreHandle) {
    let callback = CALLBACK.with(|slot| slot.borrow_mut().take().expect("armed test callback"));
    callback(owner);
}
fn callback(body: impl FnOnce(&CoreHandle) + 'static) {
    CALLBACK.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some(Box::new(body));
    });
}

#[derive(Default)]
struct Probe {
    core: Core,
    bind: Option<DataBind>,
    write: Option<WriteHook>,
    query: Option<Rc<dyn Fn()>>,
}
impl CoreCapabilities for Probe {
    fn as_data_bind(&self) -> Option<&DataBind> {
        if let Some(query) = &self.query {
            query();
        }
        self.bind.as_ref()
    }
    fn as_data_bind_mut(&mut self) -> Option<&mut DataBind> {
        self.bind.as_mut()
    }
}
impl CoreRegistryObject for Probe {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        key == 65000
    }
    fn set_uint_with_completion(&mut self, _: CoreField, _: u32, _: &mut PropertySetterCompletion) {
    }
    fn set_string_with_completion(
        &mut self,
        _: CoreField,
        _: String,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_color_with_completion(
        &mut self,
        _: CoreField,
        _: i32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_bool_with_completion(
        &mut self,
        _: CoreField,
        _: bool,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_double_with_completion(
        &mut self,
        field: CoreField,
        _: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        assert!(matches!(field, CoreField::NodeX));
        if let Some(write) = self.write.as_mut() {
            write(&self.core, completion);
        }
    }
    fn set_callback_with_completion(
        &mut self,
        _: CoreField,
        _: field_types::core_callback_type::CallbackData<'_>,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_int_with_completion(&mut self, _: CoreField, _: i32, _: &mut PropertySetterCompletion) {}
    fn get_uint(&mut self, _: CoreField) -> u32 {
        0
    }
    fn get_string(&mut self, _: CoreField) -> String {
        String::new()
    }
    fn get_color(&mut self, _: CoreField) -> i32 {
        0
    }
    fn get_bool(&mut self, _: CoreField) -> bool {
        false
    }
    fn get_double(&mut self, _: CoreField) -> f32 {
        0.0
    }
    fn get_int(&mut self, _: CoreField) -> i32 {
        0
    }
}
impl CoreObject for Probe {
    fn set_core_handle(&mut self, handle: CoreHandle) {
        self.core.set_handle(handle.clone());
        if let Some(bind) = self.bind.as_mut() {
            bind.set_core_handle(handle);
        }
    }
    fn core(&self) -> &Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }
    fn core_type(&self) -> u16 {
        65000
    }
    fn is_type_of(&self, key: u16) -> bool {
        key == 65000
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        |key| key == 65000
    }
    fn deserialize(&mut self, _: u16, _: &mut binary_reader::BinaryReader<'_>) -> bool {
        false
    }
}
fn observer(arena: &CoreArena, owner: &CoreHandle) -> CoreHandle {
    let observed = arena.insert(DataBind::new(0, KEY.into(), 0));
    owner
        .with_mut(|object| {
            observed
                .with_mut(|binding| {
                    object
                        .core_mut()
                        .add_property_observer(binding.as_data_bind_mut().unwrap());
                })
                .unwrap()
        })
        .unwrap();
    observed
}
fn dirt(owner: &CoreHandle) -> u32 {
    owner
        .with(|object| object.as_data_bind().unwrap().dirt())
        .unwrap()
}
fn finish(completion: &mut PropertySetterCompletion, consuming: bool) {
    if consuming {
        std::mem::take(completion).finish();
    } else {
        completion.finish_in_place();
    }
}
fn retired_owner() -> CoreHandle {
    let arena = CoreArena::default();
    let owner = arena.insert(Node::default());
    drop(arena.remove(&owner).unwrap());
    drop(arena);
    owner
}

#[test]
fn empty_and_equal_double_keep_existing_notification_semantics() {
    for borrowed in [false, true] {
        let mut empty = if borrowed {
            PropertySetterCompletion::borrowed_setter()
        } else {
            PropertySetterCompletion::default()
        };
        empty.finish_in_place();
        assert_eq!(empty.is_borrowed_setter(), borrowed);
        empty.finish();
    }
    let arena = CoreArena::default();
    let owner = arena.insert(Node::default());
    assert!(CoreRegistry::set_double_handle(&owner, KEY.into(), 2.0));
    let observed = observer(&arena, &owner);
    assert!(CoreRegistry::set_double_handle(&owner, KEY.into(), 2.0));
    assert_eq!(dirt(&observed), 0);
    assert!(CoreRegistry::set_double_handle(&owner, KEY.into(), 3.0));
    assert_eq!(dirt(&observed), u32::from(ComponentDirt::BINDINGS_TARGET.0));
}

#[test]
fn custom_double_completion_reenters_target_and_notifies_fresh_membership() {
    let arena = CoreArena::default();
    let owner = arena.insert(Probe {
        write: Some(Box::new(|core, completion| {
            completion.before_notification(core.handle().unwrap(), on_completion);
            completion.record(core, KEY);
        })),
        ..Probe::default()
    });
    let old = observer(&arena, &owner);
    let added = arena.insert(DataBind::new(0, KEY.into(), 0));
    let old_copy = old.clone();
    let added_copy = added.clone();
    callback(move |owner| {
        assert_eq!(dirt(&old_copy), 0, "notification must follow the callback");
        owner
            .with_mut(|object| {
                old_copy
                    .with_mut(|binding| {
                        object
                            .core_mut()
                            .remove_property_observer(binding.as_data_bind_mut().unwrap())
                    })
                    .unwrap();
                added_copy
                    .with_mut(|binding| {
                        object
                            .core_mut()
                            .add_property_observer(binding.as_data_bind_mut().unwrap())
                    })
                    .unwrap();
            })
            .unwrap();
    });
    assert!(CoreRegistry::set_double_handle(&owner, KEY.into(), 5.0));
    assert_eq!(dirt(&old), 0);
    assert_eq!(dirt(&added), u32::from(ComponentDirt::BINDINGS_TARGET.0));
}

#[test]
fn custom_setter_panic_does_not_execute_completion_and_releases_target() {
    let arena = CoreArena::default();
    let owner = arena.insert(Probe {
        write: Some(Box::new(|core, completion| {
            completion.before_notification(core.handle().unwrap(), on_completion);
            completion.record(core, KEY);
            panic!("setter panic before completion");
        })),
        ..Probe::default()
    });
    let observed = observer(&arena, &owner);
    let count = Rc::strong_count(&owner.identity);
    let invoked = Rc::new(Cell::new(false));
    let result = invoked.clone();
    callback(move |_| result.set(true));
    assert!(
        catch_unwind(AssertUnwindSafe(|| CoreRegistry::set_double_handle(
            &owner,
            KEY.into(),
            1.0
        )))
        .is_err()
    );
    assert!(!invoked.get());
    CALLBACK.with(|slot| {
        slot.borrow_mut().take();
    });
    assert_eq!(dirt(&observed), 0);
    assert_eq!(Rc::strong_count(&owner.identity), count);
    assert!(owner.with_mut(|_| ()).is_some());
}

#[test]
fn callback_panic_consumes_all_owned_resources_before_returning_to_caller() {
    for consuming in [false, true] {
        let owner = retired_owner();
        let owner_weak = Rc::downgrade(&owner.identity);
        let head = Rc::new(PropertyObservers::default());
        let head_weak = Rc::downgrade(&head);
        let observed_owner = owner_weak.clone();
        let observed_head = head_weak.clone();
        callback(move |_| {
            assert!(observed_owner.upgrade().is_some());
            assert!(
                observed_head.upgrade().is_some(),
                "notification must survive the callback"
            );
            panic!("completion callback panic");
        });
        let mut completion = PropertySetterCompletion::default();
        completion.before_notification(owner, on_completion);
        completion.notification = Some((head, KEY));
        // The completion itself intentionally survives catch_unwind. Keeping
        // notification in its caller would leak that ownership past the panic.
        assert!(catch_unwind(AssertUnwindSafe(|| finish(&mut completion, consuming))).is_err());
        assert!(owner_weak.upgrade().is_none());
        assert!(head_weak.upgrade().is_none());
        assert!(completion.before_notification.is_none() && completion.notification.is_none());
    }
}

#[test]
fn callback_owner_drops_before_observer_query_and_notification_panic_releases_head() {
    for consuming in [false, true] {
        for panic_notify in [false, true] {
            let arena = CoreArena::default();
            let owner = retired_owner();
            let owner_weak = Rc::downgrade(&owner.identity);
            let head = Rc::new(PropertyObservers::default());
            let head_weak = Rc::downgrade(&head);
            let queries = Rc::new(Cell::new(0));
            let query_count = queries.clone();
            let checked_owner = owner_weak.clone();
            let checked_head = head_weak.clone();
            let observed = arena.insert(Probe {
                bind: Some(DataBind::new(0, KEY.into(), 0)),
                query: Some(Rc::new(move || {
                    assert!(
                        checked_owner.upgrade().is_none(),
                        "callback owner drops before notify"
                    );
                    assert!(checked_head.upgrade().is_some());
                    query_count.set(query_count.get() + 1);
                    assert!(!panic_notify, "observer projection panic");
                })),
                ..Probe::default()
            });
            *head.first.borrow_mut() = Some(observed.clone());
            callback(|_| {});
            let mut completion = PropertySetterCompletion::default();
            completion.before_notification(owner, on_completion);
            completion.notification = Some((head, KEY));
            let result = catch_unwind(AssertUnwindSafe(|| finish(&mut completion, consuming)));
            assert_eq!(result.is_err(), panic_notify);
            assert!(queries.get() > 0);
            assert!(owner_weak.upgrade().is_none() && head_weak.upgrade().is_none());
            assert!(
                observed.with_mut(|_| ()).is_some(),
                "observer payload guard releases on panic"
            );
        }
    }
}

#[test]
fn last_arena_retirement_inside_callback_follows_released_double_setter() {
    let arena = CoreArena::default();
    let owner = arena.insert(Probe {
        write: Some(Box::new(|core, completion| {
            completion.before_notification(core.handle().unwrap(), on_completion);
            completion.record(core, KEY);
        })),
        ..Probe::default()
    });
    let observed = observer(&arena, &owner);
    let ran = Rc::new(Cell::new(false));
    let callback_ran = ran.clone();
    callback(move |owner| {
        assert!(owner.with_mut(|_| ()).is_some());
        drop(arena);
        assert!(!owner.is_alive());
        callback_ran.set(true);
    });
    assert!(CoreRegistry::set_double_handle(&owner, KEY.into(), 1.0));
    assert!(ran.get());
    assert!(!owner.is_alive() && !observed.is_alive());
}
