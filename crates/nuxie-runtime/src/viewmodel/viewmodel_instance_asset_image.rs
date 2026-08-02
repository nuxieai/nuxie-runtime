// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_asset_image.cpp`.
// Image asset value identity, sentinel handling, import, and clone behavior.

/// Retained safe-Rust analogue of one decoded `RenderImage*`.
#[derive(Clone)]
pub struct RuntimeViewModelImage {
    bytes: Option<Arc<[u8]>>,
    render_image: Option<Rc<dyn RenderImage>>,
}

impl RuntimeViewModelImage {
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            bytes: Some(bytes.into()),
            render_image: None,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_deref().unwrap_or_default()
    }

    pub(crate) fn from_render_image(image: Rc<dyn RenderImage>) -> Self {
        Self {
            bytes: None,
            render_image: Some(image),
        }
    }

    pub(crate) fn render_image(&self) -> Option<Rc<dyn RenderImage>> {
        self.render_image.as_ref().map(Rc::clone)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        match (
            &self.bytes,
            &other.bytes,
            &self.render_image,
            &other.render_image,
        ) {
            (Some(left), Some(right), _, _) => Arc::ptr_eq(left, right),
            (_, _, Some(left), Some(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
}

impl std::fmt::Debug for RuntimeViewModelImage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeViewModelImage")
            .field("byte_len", &self.bytes.as_ref().map_or(0, |bytes| bytes.len()))
            .field(
                "dimensions",
                &self
                    .render_image
                    .as_ref()
                    .map(|image| (image.width(), image.height())),
            )
            .finish()
    }
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeOwnedViewModelImageState {
    pub(crate) live_image: Option<RuntimeViewModelImage>,
}

#[derive(Debug)]
struct RuntimeOwnedViewModelAsset {
    property_index: usize,
    cell: RuntimeViewModelCell,
    runtime_state: Rc<RefCell<RuntimeOwnedViewModelImageState>>,
}

impl RuntimeOwnedViewModelAsset {
    fn new(property_index: usize, value: u64) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::AssetImage(
                owned_scalar_u32_payload(value),
            )),
            runtime_state: Rc::new(RefCell::new(RuntimeOwnedViewModelImageState::default())),
        }
    }

    fn value(&self) -> u64 {
        match self.cell.value() {
            RuntimeViewModelCellValue::AssetImage(value) => u64::from(value),
            _ => unreachable!("owned asset slot holds a non-asset cell"),
        }
    }

    fn set_value(&mut self, value: u64) -> bool {
        if self.value() == value {
            return false;
        }
        self.cell.set_value(RuntimeViewModelCellValue::AssetImage(
            owned_scalar_u32_payload(value),
        ));
        true
    }

    fn runtime_state(&self) -> Rc<RefCell<RuntimeOwnedViewModelImageState>> {
        Rc::clone(&self.runtime_state)
    }
}

impl Clone for RuntimeOwnedViewModelAsset {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}

fn runtime_owned_view_model_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    matches!(
                        property.type_name,
                        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage"
                    )
                    .then_some(RuntimeOwnedViewModelAsset::new(
                        property_index,
                        u64::from(u32::MAX),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_assets_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelAsset> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let value = file.view_model_instance_asset_index_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelAsset::new(property_index, value))
        })
        .collect()
}

fn runtime_owned_view_model_imported_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelAsset>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_assets_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}
