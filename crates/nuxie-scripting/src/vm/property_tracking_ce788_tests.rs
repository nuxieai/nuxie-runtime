//! Translated scripting_property_tracking_test.cpp cases at ce788b3b.
use super::*;
#[cfg(feature = "tools")]
use std::time::{Duration, Instant};

// Upstream constructs ScriptedPropertyNumber with a null model and never reads
// its numeric payload. The host Number wrapper requires a bound model, so this
// test-local userdata holds the same production lifecycle state without a
// fictitious model or callback registrations. Registration and disposal use the
// live production paths; no tracking algorithm is duplicated here.
struct UnboundProperty(Rc<ScriptedPropertyState>);
impl UserData for UnboundProperty {}

// Like makeProperties upstream, retain every userdata in a Lua table,
// independently of the Rust references used to inspect its state.
fn make_properties(lua: &Lua, count: usize) -> (Table, Vec<Rc<ScriptedPropertyState>>) {
    let retained = lua.create_table();
    let mut properties = Vec::with_capacity(count);
    for index in 1..=count {
        let state = Rc::new(ScriptedPropertyState::default());
        let property = lua.create_userdata(UnboundProperty(state)).unwrap();
        let state = property.borrow::<UnboundProperty>().unwrap().0.clone();
        register_scripted_property(lua, &state);
        properties.push(state);
        retained.raw_set(index, property).unwrap();
    }
    (retained, properties)
}

#[test]
fn untracking_in_any_order_keeps_an_objects_tracked_list_intact() {
    let lua = Lua::new();
    let context = ScriptViewModelFrameContext::for_lua(&lua);
    let object = ScriptedPropertyListenerOwner::default();
    let (retained, properties) = {
        let _scope = context.enter_owner(object.clone());
        make_properties(&lua, 2000)
    };
    assert_eq!(object.properties.borrow().len(), 2000);

    for index in (0..properties.len()).rev() {
        if index % 3 == 0 {
            properties[index].dispose();
        }
    }
    let mut remaining = Vec::new();
    for (index, property) in properties.iter().enumerate() {
        if index % 5 == 0 {
            property.dispose();
        }
        if !property.disposed() {
            remaining.push(Rc::as_ptr(property));
        }
    }
    {
        let tracked = object.properties.borrow();
        assert_eq!(tracked.len(), remaining.len());
        let mut sorted = Vec::with_capacity(tracked.len());
        for (index, property) in tracked.iter().enumerate() {
            let property = property.upgrade().unwrap();
            assert_eq!(property.track_index.get(), index);
            assert!(Rc::ptr_eq(
                &property.owner_properties.borrow().upgrade().unwrap(),
                &object.properties
            ));
            sorted.push(Rc::as_ptr(&property));
        }
        sorted.sort_unstable();
        remaining.sort_unstable();
        assert_eq!(sorted, remaining);
    }

    object.dispose();
    for property in properties {
        assert!(property.disposed());
    }
    drop(retained);
}

#[cfg(feature = "tools")]
#[test]
fn the_orphan_sweep_reaches_every_orphan_after_scattered_untracking() {
    let lua = Lua::new();
    let context = ScriptViewModelFrameContext::for_lua(&lua);
    let (retained, properties) = make_properties(&lua, 2000);
    for index in (0..properties.len()).step_by(2) {
        properties[index].dispose();
    }
    context.dispose_orphan_scripted_properties(true);
    for property in properties {
        assert!(property.disposed());
    }
    drop(retained);
}

#[cfg(feature = "tools")]
#[test]
fn disposing_many_orphans_is_linear() {
    let lua = Lua::new();
    let context = ScriptViewModelFrameContext::for_lua(&lua);
    let (retained, properties) = make_properties(&lua, 200000);
    let start = Instant::now();
    context.dispose_orphan_scripted_properties(true);
    let elapsed = start.elapsed();

    assert!(properties.first().unwrap().disposed());
    assert!(properties.last().unwrap().disposed());
    assert!(
        elapsed < Duration::from_secs(2),
        "orphan sweep took {elapsed:?}"
    );
    drop(retained);
}
