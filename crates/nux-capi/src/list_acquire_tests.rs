use super::*;
fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).unwrap();
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap()
        .key
        .int
}

fn object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .unwrap()
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn fixture(indices: &[u64]) -> Vec<u8> {
    fixture_impl(indices, false)
}
fn fixture_impl(indices: &[u64], with_artboard: bool) -> Vec<u8> {
    let mut b = b"RIVE".to_vec();
    for v in [7, 0, 3593, 0] {
        push_var_uint(&mut b, v);
    }
    object(&mut b, "Backboard", |_| {});
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Row")
    });
    object(&mut b, "ViewModelPropertyString", |b| {
        string(b, "ViewModelPropertyString", "name", "title")
    });
    for title in ["A", "B"] {
        object(&mut b, "ViewModelInstance", |b| {
            uint(b, "ViewModelInstance", "viewModelId", 0)
        });
        object(&mut b, "ViewModelInstanceString", |b| {
            uint(b, "ViewModelInstanceString", "viewModelPropertyId", 0);
            string(b, "ViewModelInstanceString", "propertyValue", title);
        });
    }
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Root")
    });
    object(&mut b, "ViewModelPropertyList", |b| {
        string(b, "ViewModelPropertyList", "name", "goals")
    });
    object(&mut b, "ViewModelPropertyViewModel", |b| {
        string(b, "ViewModelPropertyViewModel", "name", "selected");
        uint(b, "ViewModelPropertyViewModel", "viewModelReferenceId", 0);
    });
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 1)
    });
    object(&mut b, "ViewModelInstanceViewModel", |b| {
        uint(b, "ViewModelInstanceViewModel", "viewModelPropertyId", 1);
        uint(b, "ViewModelInstanceViewModel", "propertyValue", 0);
    });
    object(&mut b, "ViewModelInstanceList", |b| {
        uint(b, "ViewModelInstanceList", "viewModelPropertyId", 0)
    });
    for &index in indices {
        object(&mut b, "ViewModelInstanceListItem", |b| {
            uint(b, "ViewModelInstanceListItem", "viewModelId", 0);
            uint(b, "ViewModelInstanceListItem", "viewModelInstanceId", index);
        });
    }
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Outer")
    });
    object(&mut b, "ViewModelPropertyViewModel", |b| {
        string(b, "ViewModelPropertyViewModel", "name", "nested");
        uint(b, "ViewModelPropertyViewModel", "viewModelReferenceId", 1);
    });
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 2)
    });
    object(&mut b, "ViewModelInstanceViewModel", |b| {
        uint(b, "ViewModelInstanceViewModel", "viewModelPropertyId", 0);
        uint(b, "ViewModelInstanceViewModel", "propertyValue", 0);
    });
    // Insertion's existing adapter needs this artboard on the landing base.
    if with_artboard {
        object(&mut b, "Artboard", |b| {
            uint(b, "Artboard", "viewModelId", 0)
        });
    }
    b
}
fn view(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
unsafe fn import(bytes: &[u8], schema: usize) -> (*mut NuxFile, *mut NuxViewModelInstance) {
    unsafe {
        let mut f = ptr::null_mut();
        let mut v = ptr::null_mut();
        assert_eq!(
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut f
            ),
            NuxStatus::Ok
        );
        let schema = if schema == usize::MAX {
            (&(*f).view_model_catalog)
                .schemas
                .iter()
                .position(|s| s.name.as_ref() == b"Experience")
                .unwrap()
        } else {
            schema
        };
        assert_eq!(
            nux_view_model_instance_new_authored(f, schema, 0, &mut v),
            NuxStatus::Ok
        );
        (f, v)
    }
}
unsafe fn acquire(
    v: *const NuxViewModelInstance,
    path: &str,
    index: usize,
) -> *mut NuxViewModelInstance {
    unsafe {
        let mut item = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_list_item_acquire(v, view(path), index, &mut item),
            NuxStatus::Ok
        );
        item
    }
}
unsafe fn mutate(
    v: *mut NuxViewModelInstance,
    kind: u32,
    path: &str,
    related: *mut NuxViewModelInstance,
    index: usize,
    second_index: usize,
    text: &str,
) -> NuxStatus {
    unsafe {
        let m = NuxViewModelMutation {
            kind,
            instance: v,
            path: view(path),
            related_instance: related,
            index,
            second_index,
            bytes_value: NuxByteView {
                data: text.as_ptr(),
                len: text.len(),
            },
            ..Default::default()
        };
        let b = NuxViewModelMutationBatch {
            struct_size: std::mem::size_of::<NuxViewModelMutationBatch>() as u32,
            mutations: &m,
            mutation_count: 1,
            ..Default::default()
        };
        let mut r = ptr::null_mut();
        let status = nux_view_model_mutate(&b, &mut r);
        assert!(!r.is_null());
        assert_eq!((*r).status, status);
        nux_view_model_mutation_result_free(r);
        status
    }
}
unsafe fn title(v: *const NuxViewModelInstance) -> Vec<u8> {
    unsafe {
        (*v).instance
            .borrow()
            .string_value_by_property_name("title")
            .unwrap()
            .to_vec()
    }
}
unsafe fn titles(v: *const NuxViewModelInstance) -> Vec<Vec<u8>> {
    unsafe {
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_snapshot(v, &mut snapshot),
            NuxStatus::Ok
        );
        let values = (&(*snapshot).values)
            .iter()
            .filter(|v| v.name.as_ref() == b"title")
            .map(|v| match &v.payload {
                OwnedSnapshotPayload::Bytes(b) => b.to_vec(),
                _ => panic!("string expected"),
            })
            .collect();
        nux_view_model_snapshot_free(snapshot);
        values
    }
}
unsafe fn free(f: *mut NuxFile, handles: &[*mut NuxViewModelInstance]) {
    unsafe {
        for &v in handles {
            assert_eq!(nux_view_model_instance_free(v), NuxStatus::Ok);
        }
        assert_eq!(nux_file_free(f), NuxStatus::Ok);
    }
}
#[test]
fn list_acquire_mutates_existing_child() {
    unsafe {
        let (f, v) = import(&fixture(&[0, 1]), 1);
        let a = acquire(v, "goals", 0);
        let b = acquire(v, "goals", 1);
        let id = (*a).identity;
        assert_eq!(title(a), b"A");
        assert_eq!(title(b), b"B");
        assert_eq!(
            mutate(
                a,
                NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
                "title",
                ptr::null_mut(),
                0,
                0,
                "changed"
            ),
            NuxStatus::Ok
        );
        assert_eq!((*a).identity, id);
        assert_eq!(titles(v), [b"changed".to_vec(), b"B".to_vec()]);
        assert_eq!((*a).schema_index, 0);
        free(f, &[a, b, v]);
    }
}
#[test]
fn list_acquire_alias_share_insert_and_reference() {
    unsafe {
        let (f, v) = import(&fixture_impl(&[0, 0, 1], true), 1);
        let a = acquire(v, "goals", 0);
        let alias = acquire(v, "goals", 1);
        assert_eq!((*a).identity, (*alias).identity);
        let mut shared = ptr::null_mut();
        assert_eq!(nux_view_model_instance_share(a, &mut shared), NuxStatus::Ok);
        assert_eq!(
            mutate(
                shared,
                NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
                "title",
                ptr::null_mut(),
                0,
                0,
                "shared"
            ),
            NuxStatus::Ok
        );
        assert_eq!(title(alias), b"shared");
        assert_eq!(
            mutate(
                v,
                NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
                "goals",
                a,
                3,
                0,
                ""
            ),
            NuxStatus::Ok
        );
        let inserted = acquire(v, "goals", 3);
        assert_eq!((*inserted).identity, (*a).identity);
        assert_eq!(
            mutate(
                v,
                NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
                "selected",
                a,
                0,
                0,
                ""
            ),
            NuxStatus::Ok
        );
        assert_eq!(
            (*v).instance
                .linked_view_model_by_property_name_path("selected")
                .unwrap()
                .instance_identity(),
            (*a).identity
        );
        free(f, &[inserted, shared, alias, a, v]);
    }
}
#[test]
fn list_acquire_survives_move_remove_owner_and_file() {
    unsafe {
        let (f, v) = import(&fixture(&[0, 1]), 1);
        let a = acquire(v, "goals", 0);
        let id = (*a).identity;
        assert_eq!(
            mutate(
                v,
                NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE,
                "goals",
                ptr::null_mut(),
                0,
                1,
                ""
            ),
            NuxStatus::Ok
        );
        let moved = acquire(v, "goals", 1);
        assert_eq!((*moved).identity, id);
        assert_eq!(
            mutate(
                v,
                NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
                "goals",
                ptr::null_mut(),
                1,
                0,
                ""
            ),
            NuxStatus::Ok
        );
        free(f, &[moved, v]);
        assert_eq!(title(a), b"A");
        assert_eq!((*a).identity, id);
        assert_eq!(nux_view_model_instance_free(a), NuxStatus::Ok);
    }
}
#[test]
fn list_acquire_path_bounds_and_empty() {
    unsafe {
        let (f, v) = import(&fixture(&[]), 1);
        for (path, index, status) in [
            ("goals", 0, NuxStatus::NotFound),
            ("absent", 0, NuxStatus::NotFound),
            ("selected", 0, NuxStatus::NotFound),
            ("", 0, NuxStatus::InvalidArgument),
            ("/goals", 0, NuxStatus::InvalidArgument),
            ("goals/", 0, NuxStatus::InvalidArgument),
            ("goals", usize::MAX, NuxStatus::NotFound),
        ] {
            let mut out = v;
            assert_eq!(
                nux_view_model_instance_list_item_acquire(v, view(path), index, &mut out),
                status
            );
            assert!(out.is_null());
        }
        let mut out = v;
        assert_eq!(
            nux_view_model_instance_list_item_acquire(v, view(&"x".repeat(4097)), 0, &mut out),
            NuxStatus::LimitExceeded
        );
        assert!(out.is_null());
        let bad = [0xff];
        assert_eq!(
            nux_view_model_instance_list_item_acquire(
                v,
                NuxStringView {
                    data: bad.as_ptr().cast(),
                    len: 1
                },
                0,
                &mut out
            ),
            NuxStatus::InvalidArgument
        );
        free(f, &[v]);
    }
}
#[test]
fn list_acquire_nested_path_and_null_slot_do_not_shift() {
    unsafe {
        use nuxie::runtime::viewmodel::{
            viewmodel_instance_list::ViewModelInstanceList,
            viewmodel_instance_list_item::ViewModelInstanceListItem,
        };
        let (f, v) = import(&fixture(&[0, 1]), 2);
        let a = acquire(v, "nested/goals", 0);
        assert_eq!(title(a), b"A");
        let owner = (*v)
            .instance
            .linked_view_model_by_property_name_path("nested")
            .unwrap();
        let list = owner
            .native_handle()
            .with_downcast::<nuxie::runtime::viewmodel::viewmodel_instance::ViewModelInstance, _>(
                |v| v.property_value_named("goals"),
            )
            .flatten()
            .unwrap();
        let first = list
            .with_downcast::<ViewModelInstanceList, _>(|l| l.list_items()[0].clone())
            .unwrap();
        first
            .with_downcast_mut::<ViewModelInstanceListItem, _>(|i| i.set_view_model_instance(None));
        let mut out = a;
        assert_eq!(
            nux_view_model_instance_list_item_acquire(v, view("nested/goals"), 0, &mut out),
            NuxStatus::NotFound
        );
        assert!(out.is_null());
        let b = acquire(v, "nested/goals", 1);
        assert_eq!(title(b), b"B");
        free(f, &[a, b, v]);
    }
}
#[test]
fn list_acquire_guards_provenance_and_no_mutation() {
    unsafe {
        let (f, v) = import(&fixture(&[0, 1]), 1);
        let (f2, other) = import(&fixture(&[0, 1]), 1);
        let before = titles(v);
        use nuxie::runtime::viewmodel::viewmodel_instance_value::{
            ViewModelInstanceValueDelegate, ViewModelInstanceValueDelegateHandle,
        };
        struct Observer(Rc<Cell<usize>>);
        impl ViewModelInstanceValueDelegate for Observer {
            fn value_changed(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let notifications = Rc::new(Cell::new(0));
        let observer: ViewModelInstanceValueDelegateHandle =
            Rc::new(RefCell::new(Observer(notifications.clone())));
        let list = (*v)
            .instance
            .native_handle()
            .with(|v| {
                v.as_view_model_instance()
                    .unwrap()
                    .property_value_named("goals")
                    .unwrap()
            })
            .unwrap();
        list.with_mut(|v| {
            v.as_view_model_instance_value_mut()
                .unwrap()
                .add_delegate(&observer)
        });
        // Legacy occurrence lineage must be inherited, not silently widened.
        (*v).binding_provenance = Some(Arc::new(()));
        let a = acquire(v, "goals", 0);
        assert!(Arc::ptr_eq(
            (*a).binding_provenance.as_ref().unwrap(),
            (*v).binding_provenance.as_ref().unwrap()
        ));
        assert!(Arc::ptr_eq(&(*a).file_provenance, &(*v).file_provenance));
        assert_eq!(titles(v), before);
        assert_eq!(
            notifications.get(),
            0,
            "acquisition emits no list notification"
        );
        assert_eq!(
            mutate(
                other,
                NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
                "goals",
                a,
                0,
                0,
                ""
            ),
            NuxStatus::HandleMismatch
        );
        let mut out = v;
        for invalid in [ptr::null(), std::ptr::dangling::<NuxViewModelInstance>()] {
            let expected = if invalid.is_null() {
                NuxStatus::NullArgument
            } else {
                NuxStatus::HandleMismatch
            };
            assert_eq!(
                nux_view_model_instance_list_item_acquire(invalid, view("goals"), 0, &mut out),
                expected
            );
            assert!(out.is_null());
        }
        assert_eq!(
            nux_view_model_instance_list_item_acquire(v, view("goals"), 0, ptr::null_mut()),
            NuxStatus::NullArgument
        );
        {
            let _guard = enter_handle(v, HandleKind::ViewModel).unwrap();
            assert_eq!(
                nux_view_model_instance_list_item_acquire(v, view("goals"), 0, &mut out),
                NuxStatus::ReentrantCall
            );
            assert!(out.is_null());
        }
        {
            let _borrow = (*v).instance.borrow_mut();
            assert_eq!(
                nux_view_model_instance_list_item_acquire(v, view("goals"), 0, &mut out),
                NuxStatus::ReentrantCall
            );
        }
        let address = v as usize;
        assert_eq!(
            std::thread::spawn(move || {
                let mut out = ptr::null_mut();
                let status = nux_view_model_instance_list_item_acquire(
                    address as *const _,
                    view("goals"),
                    0,
                    &mut out,
                );
                assert!(out.is_null());
                status
            })
            .join()
            .unwrap(),
            NuxStatus::WrongThread
        );
        assert_eq!(
            mutate(
                v,
                NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
                "goals",
                ptr::null_mut(),
                0,
                0,
                ""
            ),
            NuxStatus::Ok
        );
        assert_eq!(
            notifications.get(),
            1,
            "observer control: removal does notify"
        );
        assert_eq!(nux_view_model_instance_free(a), NuxStatus::Ok);
        assert_eq!(
            nux_view_model_instance_list_item_acquire(a, view("goals"), 0, &mut out),
            NuxStatus::HandleMismatch
        );
        assert!(out.is_null());
        free(f, &[v]);
        free(f2, &[other]);
    }
}
#[test]
fn list_acquire_exact_published_f5() {
    unsafe {
        let bytes = include_bytes!("../tests/fixtures/published-goals/screen.riv");
        let (f, v) = import(bytes, usize::MAX); // Resolve delivered Experience by exact name.
        let a = acquire(v, "goals", 0);
        let b = acquire(v, "goals", 1);
        assert_eq!(title(a), b"Read");
        assert_eq!(title(b), b"Walk");
        let id = (*a).identity;
        assert_eq!(
            mutate(
                a,
                NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
                "title",
                ptr::null_mut(),
                0,
                0,
                "Rest"
            ),
            NuxStatus::Ok
        );
        assert_eq!((*a).identity, id);
        assert_eq!(title(a), b"Rest");
        assert_eq!(title(b), b"Walk");
        assert_eq!(
            mutate(
                v,
                NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
                "goals/0/title",
                ptr::null_mut(),
                0,
                0,
                "wrong"
            ),
            NuxStatus::NotFound
        );
        assert_eq!(title(a), b"Rest");
        free(f, &[a, b, v]);
    }
}
