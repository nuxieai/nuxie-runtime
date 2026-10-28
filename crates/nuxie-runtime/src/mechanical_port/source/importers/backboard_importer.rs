use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::mechanical_port::source::{
    artboard::Artboard, assets::file_asset::FileAsset, core::CoreHandle,
    file::RuntimeFileWeakHandle, status_code::StatusCode,
    viewmodel::viewmodel_instance::ViewModelInstance,
};

use super::import_stack::ImportStackObject;

// Iterative Tarjan. Components finish in reverse topological order.
fn strongly_connected_components(edges: &[Vec<usize>]) -> Vec<usize> {
    let count = edges.len();
    let mut order = vec![None; count];
    let mut lowlink = vec![0; count];
    let mut component = vec![None; count];
    let mut open = Vec::new();
    let mut calls = Vec::<(usize, usize)>::new();
    let mut next_order = 0;
    let mut next_component = 0;
    for root in 0..count {
        if order[root].is_some() {
            continue;
        }
        order[root] = Some(next_order);
        lowlink[root] = next_order;
        next_order += 1;
        open.push(root);
        calls.push((root, 0));
        while let Some(&(current, next)) = calls.last() {
            if next < edges[current].len() {
                calls.last_mut().unwrap().1 += 1;
                let neighbor = edges[current][next];
                if order[neighbor].is_none() {
                    order[neighbor] = Some(next_order);
                    lowlink[neighbor] = next_order;
                    next_order += 1;
                    open.push(neighbor);
                    calls.push((neighbor, 0));
                } else if component[neighbor].is_none() {
                    lowlink[current] = lowlink[current].min(order[neighbor].unwrap());
                }
                continue;
            }
            calls.pop();
            if let Some(&(parent, _)) = calls.last() {
                lowlink[parent] = lowlink[parent].min(lowlink[current]);
            }
            if lowlink[current] == order[current].unwrap() {
                loop {
                    let member = open.pop().unwrap();
                    component[member] = Some(next_component);
                    if member == current {
                        break;
                    }
                }
                next_component += 1;
            }
        }
    }
    component.into_iter().map(Option::unwrap).collect()
}

pub struct BackboardImporter {
    backboard: CoreHandle,
    artboard_lookup: HashMap<i32, CoreHandle>,
    artboard_referencers: Vec<CoreHandle>,
    file_assets: Vec<CoreHandle>,
    file_asset_referencers: Vec<CoreHandle>,
    data_converters: Vec<CoreHandle>,
    data_converter_referencers: Vec<CoreHandle>,
    data_converter_group_item_referencers: Vec<CoreHandle>,
    interpolators: Vec<CoreHandle>,
    physics: Vec<CoreHandle>,
    next_artboard_id: i32,
    file: Option<RuntimeFileWeakHandle>,
    file_view_models: Option<Rc<RefCell<Vec<CoreHandle>>>>,
    file_view_model_instances: Option<Rc<RefCell<Vec<CoreHandle>>>>,
}

impl BackboardImporter {
    pub fn new(backboard: CoreHandle) -> Self {
        Self {
            backboard,
            artboard_lookup: HashMap::new(),
            artboard_referencers: Vec::new(),
            file_assets: Vec::new(),
            file_asset_referencers: Vec::new(),
            data_converters: Vec::new(),
            data_converter_referencers: Vec::new(),
            data_converter_group_item_referencers: Vec::new(),
            interpolators: Vec::new(),
            physics: Vec::new(),
            next_artboard_id: 0,
            file: None,
            file_view_models: None,
            file_view_model_instances: None,
        }
    }

    pub fn add_artboard_referencer(&mut self, artboard: CoreHandle) {
        self.artboard_referencers.push(artboard);
    }

    pub fn add_file_asset(&mut self, asset: &mut FileAsset) {
        let handle = asset
            .handle()
            .expect("imported FileAsset has an arena owner");
        self.file_assets.push(handle.clone());
        let mut ids = HashSet::new();
        let mut next_id = 1u32;
        let mut ensure_unique_id = |file_asset: &mut FileAsset| {
            let asset_id = file_asset.asset_id();
            if ids.contains(&asset_id) {
                file_asset.set_asset_id(next_id);
            } else {
                ids.insert(asset_id);
                if asset_id >= next_id {
                    next_id = asset_id.wrapping_add(1);
                }
            }
        };
        for file_asset in &self.file_assets {
            if *file_asset == handle {
                ensure_unique_id(asset);
            } else {
                file_asset
                    .with_mut(|asset| {
                        ensure_unique_id(
                            asset
                                .as_file_asset_mut()
                                .expect("BackboardImporter assets remain FileAsset-derived")
                                .file_asset_base_mut(),
                        );
                    })
                    .expect("BackboardImporter retains live FileAssets");
            }
        }
    }

    pub fn add_file_asset_referencer(&mut self, referencer: CoreHandle) {
        self.file_asset_referencers.push(referencer);
    }

    pub fn add_artboard(&mut self, artboard: &mut Artboard) {
        #[cfg(feature = "tools")]
        artboard.set_artboard_id(self.next_artboard_id as u16);
        let artboard = crate::mechanical_port::source::core::CoreObject::core(artboard)
            .handle()
            .expect("imported Artboard owner");
        self.artboard_lookup.insert(self.next_artboard_id, artboard);
        self.next_artboard_id += 1;
    }

    pub fn add_missing_artboard(&mut self) {
        self.next_artboard_id += 1;
    }

    pub fn add_data_converter(&mut self, converter: CoreHandle) {
        self.data_converters.push(converter);
    }

    pub fn last_data_converter(&self) -> Option<CoreHandle> {
        self.data_converters.last().cloned()
    }

    pub fn add_data_converter_referencer(&mut self, data_bind: CoreHandle) {
        self.data_converter_referencers.push(data_bind);
    }

    pub fn add_data_converter_group_item_referencer(&mut self, item: CoreHandle) {
        self.data_converter_group_item_referencers.push(item);
    }

    pub fn add_interpolator(
        &mut self,
        interpolator: &mut dyn crate::mechanical_port::source::core::CoreObject,
    ) {
        // Import is already executing on this owner. Invoke the real virtual
        // initialize before retaining its handle, without reborrowing its slot.
        let initialized = interpolator.keyframe_interpolator_initialize();
        assert!(
            initialized,
            "BackboardImporter interpolators must implement KeyFrameInterpolator"
        );
        self.interpolators.push(
            interpolator
                .core()
                .handle()
                .expect("imported interpolator owner"),
        );
    }

    pub fn add_physics(&mut self, physics: CoreHandle) {
        self.physics.push(physics);
    }

    /// Retain an already initialized file-global interpolator for partial import.
    pub fn seed_interpolator(&mut self, interpolator: CoreHandle) {
        self.interpolators.push(interpolator);
    }

    pub fn add_view_model_instance(&mut self, instance: &mut ViewModelInstance) {
        let Some(models) = self.file_view_models.as_ref() else {
            return;
        };
        let view_model = models
            .borrow()
            .get(instance.base.view_model_id() as usize)
            .cloned();
        if let Some(view_model) = view_model {
            view_model
                .with_mut(|view_model| {
                    view_model
                        .as_view_model_mut()
                        .expect("File view model handles remain ViewModel-derived")
                        .add_instance(instance);
                })
                .expect("File retains live ViewModels");
        }
    }

    pub(crate) fn add_file_view_model_instance(&mut self, instance: CoreHandle) {
        if let Some(instances) = &self.file_view_model_instances {
            instances.borrow_mut().push(instance);
        }
    }

    pub fn physics(&self) -> Vec<CoreHandle> {
        self.physics.clone()
    }

    pub fn assets(&mut self) -> &mut Vec<CoreHandle> {
        &mut self.file_assets
    }

    pub(crate) fn set_file(
        &mut self,
        file: RuntimeFileWeakHandle,
        view_models: Rc<RefCell<Vec<CoreHandle>>>,
        view_model_instances: Rc<RefCell<Vec<CoreHandle>>>,
    ) {
        self.file = Some(file);
        // These are the File's canonical lists, shared only to permit synchronous
        // registration while File::read and the imported instance are borrowed.
        self.file_view_models = Some(view_models);
        self.file_view_model_instances = Some(view_model_instances);
    }

    pub fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.clone()
    }

    pub fn backboard(&self) -> CoreHandle {
        self.backboard.clone()
    }

    // Resolve owned bindings before any use clones their converter. Same-SCC
    // edges deliberately retain no converter, avoiding unbounded cloning.
    fn resolve_converter_data_binds(&self) -> HashSet<CoreHandle> {
        use crate::source::data_bind::converters::data_converter_group::DataConverterGroup;
        let count = self.data_converters.len();
        let indices: HashMap<_, _> = self
            .data_converters
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, converter)| (converter, i))
            .collect();
        let converter_index = |bind: &CoreHandle| {
            bind.with(|bind| bind.data_bind_converter_id())
                .flatten()
                .map(|index| index as usize)
                .filter(|index| *index < count)
        };
        let mut edges = vec![Vec::new(); count];
        for (i, converter) in self.data_converters.iter().enumerate() {
            let binds = converter
                .data_bind_container()
                .expect("converter container");
            for bind in binds.data_binds().iter() {
                if let Some(to) = converter_index(bind) {
                    edges[i].push(to);
                }
            }
            if let Some(items) =
                converter.with_downcast::<DataConverterGroup, _>(|group| group.items().to_vec())
            {
                for item in items {
                    if let Some(converter) = item
                        .with(|item| item.as_data_converter_group_item().unwrap().converter())
                        .flatten()
                        && let Some(to) = indices.get(&converter)
                    {
                        edges[i].push(*to);
                    }
                }
            }
        }
        let component = strongly_connected_components(&edges);
        let mut order: Vec<_> = (0..count).collect();
        order.sort_by_key(|i| component[*i]);
        let mut resolved = HashSet::new();
        for i in order {
            let binds = self.data_converters[i]
                .data_bind_container()
                .expect("converter container")
                .data_binds()
                .to_vec();
            for bind in binds {
                resolved.insert(bind.clone());
                let Some(to) = converter_index(&bind) else {
                    continue;
                };
                if component[to] == component[i]
                    || bind
                        .with(|bind| bind.as_data_bind().unwrap().converter().is_some())
                        .unwrap()
                {
                    continue;
                }
                let converter = self.data_converters[to]
                    .clone_occurrence()
                    .expect("converter clone");
                bind.with_mut(|bind| {
                    bind.as_data_bind_mut()
                        .unwrap()
                        .set_converter(Some(converter))
                });
            }
        }
        resolved
    }

    fn cyclic_artboard_references(&self) -> Vec<bool> {
        let mut nodes = HashMap::<CoreHandle, usize>::new();
        let mut references = vec![None; self.artboard_referencers.len()];
        for (index, referencer) in self.artboard_referencers.iter().enumerate() {
            let Some((nesting, target)) = referencer
                .with(|referencer| {
                    Some((
                        referencer.artboard_referencer_nesting_artboard()?,
                        referencer.artboard_referencer_referenced_artboard_id()?,
                    ))
                })
                .flatten()
            else {
                continue;
            };
            let Some(nested) = self.artboard_lookup.get(&target) else {
                continue;
            };
            let next = nodes.len();
            let from = *nodes.entry(nesting).or_insert(next);
            let next = nodes.len();
            let to = *nodes.entry(nested.clone()).or_insert(next);
            references[index] = Some((from, to));
        }
        let count = nodes.len();
        let mut edges = vec![Vec::new(); count];
        for &(from, to) in references.iter().flatten() {
            edges[from].push(to);
        }
        let component = strongly_connected_components(&edges);
        references
            .into_iter()
            .map(|edge| edge.is_some_and(|(from, to)| component[from] == component[to]))
            .collect()
    }
}

impl ImportStackObject for BackboardImporter {
    fn resolve(&mut self) -> StatusCode {
        let cyclic = self.cyclic_artboard_references();
        for (index, referencer) in self.artboard_referencers.iter().enumerate() {
            if cyclic[index] {
                continue;
            }
            let referenced_artboard_id = referencer
                .with(|referencer| referencer.artboard_referencer_referenced_artboard_id())
                .expect("BackboardImporter retains live ArtboardReferencers")
                .expect("registered ArtboardReferencers expose their referenced artboard id");
            if let Some(artboard) = self.artboard_lookup.get(&referenced_artboard_id) {
                let assigned = referencer
                    .with_mut(|referencer| {
                        referencer.artboard_referencer_set_referenced_artboard(artboard.clone())
                    })
                    .expect("BackboardImporter retains live ArtboardReferencers");
                assert!(
                    assigned,
                    "an ArtboardReferencer exposing an id must accept its resolved Artboard"
                );
            }
        }

        for referencer in &self.file_asset_referencers {
            if let Some(valid) = referencer.with_downcast_mut::<crate::video::Video, _>(|video| {
                self.file_assets
                    .get(video.asset_id() as usize)
                    .is_some_and(|asset| asset.is_type_of(crate::video::VideoAsset::TYPE_KEY))
                    && video.playback.settings().valid()
                    && video.captions_valid()
                    && video.resolve_poster(&self.file_assets)
            }) {
                if !valid {
                    return StatusCode::InvalidObject;
                }
            }
            let index = referencer
                .with(|referencer| referencer.file_asset_referencer_asset_id())
                .expect("BackboardImporter retains live FileAssetReferencers")
                .expect("registered FileAssetReferencers expose their asset id")
                as usize;
            if index >= self.file_assets.len() {
                continue;
            }
            let assigned = crate::mechanical_port::source::generated::core_registry::file_asset_referencer_set_asset_handle(referencer, self.file_assets[index].clone());
            assert!(
                assigned,
                "a FileAssetReferencer exposing an asset id must accept its resolved FileAsset"
            );
        }

        for converter in &self.data_converters {
            let Some(interpolator_id) = converter
                .with(|converter| converter.data_converter_interpolator_id())
                .expect("BackboardImporter retains live DataConverters")
            else {
                continue;
            };
            let index = interpolator_id as usize;
            if index != usize::MAX && index < self.interpolators.len() {
                let assigned = converter
                    .with_mut(|converter| {
                        converter.data_converter_set_interpolator(self.interpolators[index].clone())
                    })
                    .expect("BackboardImporter retains live DataConverters");
                assert!(
                    assigned,
                    "a converter exposing an interpolator id must accept its interpolator"
                );
            }
        }

        for referencer in &self.data_converter_group_item_referencers {
            let index = referencer
                .with(|referencer| referencer.data_converter_group_item_converter_id())
                .expect("BackboardImporter retains live DataConverterGroupItems")
                .expect("registered DataConverterGroupItems expose their converter id")
                as usize;
            if index >= self.data_converters.len() {
                continue;
            }
            let assigned = referencer
                .with_mut(|referencer| {
                    referencer.data_converter_group_item_set_converter(
                        self.data_converters[index].clone(),
                    )
                })
                .expect("BackboardImporter retains live DataConverterGroupItems");
            assert!(
                assigned,
                "a DataConverterGroupItem exposing an id must accept its converter"
            );
        }

        let converter_data_binds = self.resolve_converter_data_binds();
        for referencer in &self.data_converter_referencers {
            if converter_data_binds.contains(referencer) {
                continue;
            }
            let index = referencer
                .with(|referencer| referencer.data_bind_converter_id())
                .expect("BackboardImporter retains live DataBinds")
                .expect("registered DataBinds expose their converter id")
                as usize;
            if index >= self.data_converters.len() {
                continue;
            }
            let converter = self.data_converters[index]
                .clone_occurrence()
                .expect("registered DataConverters must support pinned Core cloning");
            let assigned = referencer
                .with_mut(|referencer| referencer.data_bind_set_converter(converter))
                .expect("BackboardImporter retains live DataBinds");
            assert!(
                assigned,
                "a DataBind exposing a converter id must accept its cloned converter"
            );
        }
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod recursive_converter_tests {
    use super::*;
    use crate::source::{
        backboard::Backboard,
        core::CoreArena,
        data_bind::{
            converters::{
                data_converter_group::DataConverterGroup,
                data_converter_group_item::DataConverterGroupItem,
                data_converter_rounder::DataConverterRounder,
            },
            data_bind::DataBind,
        },
        generated::core_registry::CoreRegistry,
    };
    use crate::{File, RuntimeFactoryHandle, RuntimeFileHandle};
    use nuxie_render_api::{PersistentFactory, RecordingFactory};

    fn file() -> RuntimeFileHandle {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        RuntimeFileHandle::new(File::new(
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
        ))
    }

    fn bind(
        arena: &CoreArena,
        file: &RuntimeFileHandle,
        owner: &CoreHandle,
        index: u32,
    ) -> CoreHandle {
        let mut bind = DataBind::default();
        bind.set_file(file.downgrade());
        let bind = arena.insert(bind);
        assert!(CoreRegistry::set_uint_handle(&bind, 660, index));
        owner
            .data_bind_container()
            .unwrap()
            .add_data_bind(bind.clone());
        bind.with_mut(|bind| {
            bind.as_data_bind_mut()
                .unwrap()
                .set_target(Some(owner.clone()))
        });
        bind
    }

    fn converter(bind: &CoreHandle) -> Option<CoreHandle> {
        bind.with(|bind| bind.as_data_bind().unwrap().converter())
            .flatten()
    }

    #[test]
    fn resolves_nested_converter_binds_before_cloning_and_keeps_partial_import_identity() {
        let arena = CoreArena::default();
        let file = file();
        let mut importer = BackboardImporter::new(arena.insert(Backboard::default()));
        let outer = arena.insert(DataConverterRounder::default());
        let middle = arena.insert(DataConverterRounder::default());
        let leaf = arena.insert(DataConverterRounder::default());
        importer.data_converters = vec![outer.clone(), middle.clone(), leaf.clone()];
        let outer_bind = bind(&arena, &file, &outer, 1);
        let middle_bind = bind(&arena, &file, &middle, 2);
        let resolved = importer.resolve_converter_data_binds();
        assert_eq!(resolved.len(), 2);
        let middle_clone = converter(&outer_bind).unwrap();
        assert_ne!(middle_clone, middle);
        let nested_bind = middle_clone.data_bind_container().unwrap().data_binds()[0].clone();
        let leaf_clone = converter(&nested_bind).unwrap();
        assert_ne!(leaf_clone, converter(&middle_bind).unwrap());
        assert_eq!(
            nested_bind.with(|bind| bind.as_data_bind().unwrap().target()),
            Some(Some(middle_clone.clone()))
        );
        importer.resolve_converter_data_binds();
        assert_eq!(converter(&outer_bind), Some(middle_clone));
    }

    #[test]
    fn converter_cycles_drop_only_the_cyclic_bind_edges() {
        let arena = CoreArena::default();
        let file = file();
        let mut importer = BackboardImporter::new(arena.insert(Backboard::default()));
        let first = arena.insert(DataConverterRounder::default());
        let second = arena.insert(DataConverterRounder::default());
        let leaf = arena.insert(DataConverterRounder::default());
        importer.data_converters = vec![first.clone(), second.clone(), leaf];
        let first_cycle = bind(&arena, &file, &first, 1);
        let second_cycle = bind(&arena, &file, &second, 0);
        let outside = bind(&arena, &file, &second, 2);
        let invalid = bind(&arena, &file, &first, 99);
        assert_eq!(importer.resolve_converter_data_binds().len(), 4);
        assert!(converter(&first_cycle).is_none());
        assert!(converter(&second_cycle).is_none());
        assert!(converter(&invalid).is_none());
        assert!(converter(&outside).is_some());
    }

    #[test]
    fn converter_cycle_detection_includes_group_item_edges() {
        let arena = CoreArena::default();
        let file = file();
        let mut importer = BackboardImporter::new(arena.insert(Backboard::default()));
        let group = arena.insert(DataConverterGroup::default());
        let child = arena.insert(DataConverterRounder::default());
        let mut item = DataConverterGroupItem::default();
        item.set_converter(Some(child.clone()));
        group
            .with_downcast_mut::<DataConverterGroup, _>(|group| group.add_item(arena.insert(item)));
        importer.data_converters = vec![group, child.clone()];
        let cycle = bind(&arena, &file, &child, 0);
        importer.resolve_converter_data_binds();
        assert!(converter(&cycle).is_none());
    }
}
