// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_runtime.cpp`.
// Retains one concrete instance and lazily caches exact typed wrappers.

#[derive(Debug)]
struct ViewModelInstanceRuntimeInner {
    file: Rc<RuntimeFile>,
    instance: RuntimeOwnedViewModelHandle,
    properties: RefCell<BTreeMap<String, ViewModelInstanceRuntimeProperty>>,
    view_model_instances: RefCell<BTreeMap<String, ViewModelInstanceRuntime>>,
}

#[derive(Debug, Clone)]
pub struct ViewModelInstanceRuntime {
    inner: Rc<ViewModelInstanceRuntimeInner>,
}

impl ViewModelInstanceRuntime {
    pub fn new(file: Rc<RuntimeFile>, instance: RuntimeOwnedViewModelHandle) -> Self {
        Self::from_handle(file, instance)
    }

    fn from_handle(file: Rc<RuntimeFile>, instance: RuntimeOwnedViewModelHandle) -> Self {
        Self {
            inner: Rc::new(ViewModelInstanceRuntimeInner {
                file,
                instance,
                properties: RefCell::new(BTreeMap::new()),
                view_model_instances: RefCell::new(BTreeMap::new()),
            }),
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn name(&self) -> String {
        self.inner.instance.borrow().name().to_owned()
    }

    pub fn view_model_name(&self) -> String {
        self.inner
            .file
            .view_model(self.inner.instance.borrow().view_model_index())
            .and_then(|view_model| view_model.object.string_property("name"))
            .unwrap_or_default()
            .to_owned()
    }

    pub fn property_count(&self) -> usize {
        self.inner.instance.borrow().property_value_count()
    }

    pub fn handle(&self) -> &RuntimeOwnedViewModelHandle {
        &self.inner.instance
    }

    pub fn properties(&self) -> Vec<ViewModelRuntimeProperty> {
        ViewModelRuntime::build_properties_data(
            &self.inner.file,
            self.inner.instance.borrow().view_model_index(),
        )
    }

    fn runtime_for_property_path(&self, path: &str) -> Option<(Self, String)> {
        if path.is_empty() {
            return None;
        }
        let mut segments = path.split('/').collect::<Vec<_>>();
        if segments.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_name = segments.pop()?.to_owned();
        let mut runtime = self.clone();
        for segment in segments {
            runtime = runtime.instance_runtime(segment)?;
        }
        Some((runtime, property_name))
    }

    fn instance_runtime(&self, name: &str) -> Option<Self> {
        if let Some(runtime) = self.inner.view_model_instances.borrow().get(name) {
            return Some(runtime.clone());
        }
        let instance = self
            .inner
            .instance
            .linked_view_model_by_property_name_path(name)?;
        let runtime = Self::from_handle(Rc::clone(&self.inner.file), instance);
        self.inner
            .view_model_instances
            .borrow_mut()
            .insert(name.to_owned(), runtime.clone());
        Some(runtime)
    }

    pub fn view_model_instance_at_path(&self, path: &str) -> Option<Self> {
        if path.is_empty() || path.split('/').any(|segment| segment.is_empty()) {
            return None;
        }
        let mut runtime = self.clone();
        for segment in path.split('/') {
            runtime = runtime.instance_runtime(segment)?;
        }
        Some(runtime)
    }

    fn property_index_and_cell(
        &self,
        name: &str,
    ) -> Option<(usize, RuntimeViewModelCell)> {
        let instance = self.inner.instance.borrow();
        let property_index = instance.property_index_by_name(name)?;
        let cell = instance.cell_by_property_path(&[property_index])?;
        Some((property_index, cell))
    }

    fn property_direct(&self, name: &str) -> Option<ViewModelInstanceRuntimeProperty> {
        if let Some(property) = self.inner.properties.borrow().get(name) {
            return Some(property.clone());
        }
        let (property_index, cell) = self.property_index_and_cell(name)?;
        let property = match cell.value() {
            RuntimeViewModelCellValue::Number(_) => ViewModelInstanceRuntimeProperty::Number(
                ViewModelInstanceNumberRuntime::new(name, cell),
            ),
            RuntimeViewModelCellValue::String(_) => ViewModelInstanceRuntimeProperty::String(
                ViewModelInstanceStringRuntime::new(name, cell),
            ),
            RuntimeViewModelCellValue::Boolean(_) => ViewModelInstanceRuntimeProperty::Boolean(
                ViewModelInstanceBooleanRuntime::new(name, cell),
            ),
            RuntimeViewModelCellValue::Color(_) => ViewModelInstanceRuntimeProperty::Color(
                ViewModelInstanceColorRuntime::new(name, cell),
            ),
            RuntimeViewModelCellValue::Enum(_) => {
                ViewModelInstanceRuntimeProperty::Enum(ViewModelInstanceEnumRuntime::new(
                    name,
                    cell,
                    Rc::clone(&self.inner.file),
                    self.inner.instance.borrow().view_model_index(),
                    property_index,
                ))
            }
            RuntimeViewModelCellValue::Trigger(_) => ViewModelInstanceRuntimeProperty::Trigger(
                ViewModelInstanceTriggerRuntime::new(name, cell),
            ),
            RuntimeViewModelCellValue::SymbolListIndex(_) => {
                ViewModelInstanceRuntimeProperty::ListIndex(
                    ViewModelInstanceListIndexRuntime::new(name, cell),
                )
            }
            RuntimeViewModelCellValue::List => {
                ViewModelInstanceRuntimeProperty::List(ViewModelInstanceListRuntime::new(
                    name,
                    cell,
                    Rc::clone(&self.inner.file),
                    self.inner.instance.clone(),
                    vec![property_index],
                ))
            }
            RuntimeViewModelCellValue::AssetImage(_) => {
                let runtime_state = self
                    .inner
                    .instance
                    .borrow()
                    .image_runtime_state_by_property_index(property_index)?;
                ViewModelInstanceRuntimeProperty::AssetImage(
                    ViewModelInstanceAssetImageRuntime::new(name, cell, runtime_state),
                )
            }
            RuntimeViewModelCellValue::AssetFont(_) => {
                ViewModelInstanceRuntimeProperty::AssetFont(
                    ViewModelInstanceAssetFontRuntime::new(name, cell),
                )
            }
            RuntimeViewModelCellValue::AssetBlob(_) => {
                ViewModelInstanceRuntimeProperty::AssetBlob(
                    ViewModelInstanceAssetBlobRuntime::new(name, cell),
                )
            }
            RuntimeViewModelCellValue::Artboard(_) => {
                let runtime_state = self
                    .inner
                    .instance
                    .borrow()
                    .artboard_runtime_state_by_property_index(property_index)?;
                ViewModelInstanceRuntimeProperty::Artboard(
                    ViewModelInstanceArtboardRuntime::new(
                        name,
                        cell,
                        runtime_state,
                    ),
                )
            }
            RuntimeViewModelCellValue::ViewModel => return None,
        };
        self.inner
            .properties
            .borrow_mut()
            .insert(name.to_owned(), property.clone());
        Some(property)
    }

    pub fn property(&self, path: &str) -> Option<ViewModelInstanceRuntimeProperty> {
        let (runtime, property_name) = self.runtime_for_property_path(path)?;
        runtime.property_direct(&property_name)
    }

    pub fn property_number(&self, path: &str) -> Option<ViewModelInstanceNumberRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_string(&self, path: &str) -> Option<ViewModelInstanceStringRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_boolean(&self, path: &str) -> Option<ViewModelInstanceBooleanRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::Boolean(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_color(&self, path: &str) -> Option<ViewModelInstanceColorRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::Color(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_enum(&self, path: &str) -> Option<ViewModelInstanceEnumRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::Enum(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_trigger(&self, path: &str) -> Option<ViewModelInstanceTriggerRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::Trigger(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_list_index(&self, path: &str) -> Option<ViewModelInstanceListIndexRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::ListIndex(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_list(&self, path: &str) -> Option<ViewModelInstanceListRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::List(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_image(&self, path: &str) -> Option<ViewModelInstanceAssetImageRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::AssetImage(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_font(&self, path: &str) -> Option<ViewModelInstanceAssetFontRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::AssetFont(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_blob(&self, path: &str) -> Option<ViewModelInstanceAssetBlobRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::AssetBlob(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_artboard(&self, path: &str) -> Option<ViewModelInstanceArtboardRuntime> {
        match self.property(path)? {
            ViewModelInstanceRuntimeProperty::Artboard(value) => Some(value),
            _ => None,
        }
    }

    pub fn property_view_model(&self, path: &str) -> Option<Self> {
        let (runtime, property_name) = self.runtime_for_property_path(path)?;
        runtime.instance_runtime(&property_name)
    }

    pub fn replace_view_model(&self, path: &str, value: &Self) -> bool {
        let Some((runtime, property_name)) = self.runtime_for_property_path(path) else {
            return false;
        };
        runtime.replace_view_model_by_name(&property_name, value)
    }

    fn replace_view_model_by_name(&self, name: &str, value: &Self) -> bool {
        if self
            .inner
            .instance
            .link_view_model_by_property_name_path(name, value.handle())
            != Ok(true)
        {
            return false;
        }
        let is_stored = self
            .inner
            .view_model_instances
            .borrow()
            .values()
            .any(|stored| stored.ptr_eq(value));
        if !is_stored {
            self.inner
                .view_model_instances
                .borrow_mut()
                .insert(name.to_owned(), value.clone());
        }
        true
    }
}

#[cfg(test)]
mod viewmodel_instance_runtime_identity_tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
    use nuxie_schema::definition_by_name;

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value,
        }
    }

    pub(super) fn runtime_family_file() -> Rc<RuntimeFile> {
        Rc::new(
            RuntimeFile::from_authoring_records(vec![
                record("Backboard", Vec::new()),
                record(
                    "ViewModel",
                    vec![property(
                        "ViewModel",
                        "name",
                        AuthoringValue::String("Root".to_owned()),
                    )],
                ),
                record(
                    "ViewModelPropertyNumber",
                    vec![property(
                        "ViewModelPropertyNumber",
                        "name",
                        AuthoringValue::String("count".to_owned()),
                    )],
                ),
                record(
                    "ViewModelPropertyList",
                    vec![property(
                        "ViewModelPropertyList",
                        "name",
                        AuthoringValue::String("items".to_owned()),
                    )],
                ),
                record(
                    "ViewModelPropertyViewModel",
                    vec![
                        property(
                            "ViewModelPropertyViewModel",
                            "name",
                            AuthoringValue::String("child".to_owned()),
                        ),
                        property(
                            "ViewModelPropertyViewModel",
                            "viewModelReferenceId",
                            AuthoringValue::Uint(1),
                        ),
                    ],
                ),
                record(
                    "ViewModel",
                    vec![property(
                        "ViewModel",
                        "name",
                        AuthoringValue::String("Child".to_owned()),
                    )],
                ),
                record(
                    "ViewModelPropertyNumber",
                    vec![property(
                        "ViewModelPropertyNumber",
                        "name",
                        AuthoringValue::String("value".to_owned()),
                    )],
                ),
            ])
            .expect("runtime facade fixture"),
        )
    }

    fn runtime_asset_file() -> Rc<RuntimeFile> {
        Rc::new(
            RuntimeFile::from_authoring_records(vec![
                record("Backboard", Vec::new()),
                record(
                    "ViewModel",
                    vec![property(
                        "ViewModel",
                        "name",
                        AuthoringValue::String("Assets".to_owned()),
                    )],
                ),
                record(
                    "ViewModelPropertyAssetImage",
                    vec![property(
                        "ViewModelPropertyAssetImage",
                        "name",
                        AuthoringValue::String("image".to_owned()),
                    )],
                ),
                record(
                    "ViewModelPropertyArtboard",
                    vec![property(
                        "ViewModelPropertyArtboard",
                        "name",
                        AuthoringValue::String("artboard".to_owned()),
                    )],
                ),
            ])
            .expect("runtime asset fixture"),
        )
    }

    #[test]
    fn repeated_typed_and_nested_lookups_preserve_wrapper_identity() {
        let file = runtime_family_file();
        let root = ViewModelRuntime::new(Rc::clone(&file), 0)
            .expect("root runtime")
            .create_instance()
            .expect("root instance");

        let first = root.property_number("count").expect("number");
        let second = root.property_number("count").expect("number");
        assert!(first.ptr_eq(&second));
        assert!(root.property_string("count").is_none());
        assert!(first.ptr_eq(&root.property_number("count").expect("number")));

        let child = root.property_view_model("child").expect("child");
        assert!(child.ptr_eq(&root.property_view_model("child").expect("child")));
        let nested = root.property_number("child/value").expect("nested number");
        assert!(nested.ptr_eq(
            &root
                .property_number("child/value")
                .expect("same nested number")
        ));
        assert!(nested.ptr_eq(&child.property_number("value").expect("child number")));
    }

    #[test]
    fn replacement_and_list_mutation_keep_concrete_runtime_identity() {
        let file = runtime_family_file();
        let root = ViewModelRuntime::new(Rc::clone(&file), 0)
            .expect("root runtime")
            .create_instance()
            .expect("root instance");
        let child_model = ViewModelRuntime::new(Rc::clone(&file), 1).expect("child runtime");
        let replacement = child_model.create_instance().expect("replacement");

        let _old = root.property_view_model("child").expect("old child");
        assert!(root.replace_view_model("child", &replacement));
        assert!(
            root.property_view_model("child")
                .expect("replacement child")
                .ptr_eq(&replacement)
        );

        let list = root.property_list("items").expect("list");
        assert!(list.ptr_eq(&root.property_list("items").expect("same list")));
        assert!(list.add_instance(&replacement));
        assert!(list.add_instance(&replacement));
        assert_eq!(list.size(), 2);
        assert!(
            list.instance_at(0)
                .expect("first occurrence")
                .ptr_eq(&replacement)
        );
        assert!(
            list.instance_at(1)
                .expect("second occurrence")
                .ptr_eq(&replacement)
        );
        assert!(list.swap(0, 1));
        assert!(
            list.instance_at(0)
                .expect("swapped occurrence")
                .ptr_eq(&replacement)
        );
        assert!(list.remove_instance(&replacement));
        assert_eq!(list.size(), 0);
        assert!(list.instance_at(0).is_none());
    }

    #[test]
    fn dropping_wrappers_never_drops_the_underlying_property_or_instance() {
        let file = runtime_family_file();
        let root = ViewModelRuntime::new(file, 0)
            .expect("root runtime")
            .create_instance()
            .expect("root instance");
        {
            let value = root.property_number("count").expect("number");
            assert!(value.set_value(41.0));
        }
        assert_eq!(
            root.handle().borrow().number_value_by_property_name("count"),
            Some(41.0)
        );
        assert_eq!(
            root.property_number("count").expect("cached number").value(),
            41.0
        );
    }

    #[test]
    fn live_asset_state_survives_facade_reacquisition() {
        let file = runtime_asset_file();
        let runtime = ViewModelRuntime::new(Rc::clone(&file), 0)
            .expect("asset runtime")
            .create_instance()
            .expect("asset instance");
        let handle = runtime.handle().clone();

        let image_value = RuntimeViewModelImage::new(Arc::<[u8]>::from(&b"pixels"[..]));
        assert!(
            runtime
                .property_image("image")
                .expect("image")
                .set_value(Some(image_value.clone()))
        );
        let artboard_value = RuntimeBindableArtboard::new("Nested");
        assert!(
            runtime
                .property_artboard("artboard")
                .expect("artboard")
                .set_value(Some(artboard_value.clone()))
        );
        drop(runtime);

        let reacquired = ViewModelInstanceRuntime::new(file, handle);
        assert!(
            reacquired
                .property_image("image")
                .expect("reacquired image")
                .value()
                .expect("live image")
                .ptr_eq(&image_value)
        );
        assert_eq!(
            reacquired
                .property_artboard("artboard")
                .expect("reacquired artboard")
                .artboard_name(),
            "Nested"
        );
    }

    #[test]
    fn indexed_list_insert_preserves_cpp_parent_registration_order() {
        let file = runtime_family_file();
        let root = ViewModelRuntime::new(Rc::clone(&file), 0)
            .expect("root runtime")
            .create_instance()
            .expect("root instance");
        let child_model = ViewModelRuntime::new(file, 1).expect("child runtime");
        let appended = child_model.create_instance().expect("appended child");
        let inserted = child_model.create_instance().expect("inserted child");
        let list = root.property_list("items").expect("list");
        let dependent = RuntimeCellDirtSink::new();
        root.handle().add_rebind_dependent(&dependent);

        assert!(list.add_instance(&appended));
        dependent.take_dirt();
        appended.handle().borrow_mut().mark_structurally_mutated();
        assert!(
            dependent
                .take_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );

        assert!(list.add_instance_at(&inserted, 0));
        dependent.take_dirt();
        inserted.handle().borrow_mut().mark_structurally_mutated();
        assert!(dependent.take_dirt().is_empty());
    }
}
