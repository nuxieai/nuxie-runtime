//! Exact executable ports of pinned `image_asset_test.cpp`.

use std::{collections::BTreeMap, path::PathBuf};

use nuxie::{
    CoreHandle, File, FileAssetLoader, FileAssetLoaderRef, PersistentFactory,
    RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    runtime::{artboard::Artboard, assets::image_asset::ImageAsset, shapes::image::Image},
};
use nuxie_render_api::RecordingFactory;

fn pinned_asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

struct ExternalImageLoader {
    images: BTreeMap<String, Vec<u8>>,
}

impl FileAssetLoader for ExternalImageLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        in_band_bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        asset
            .with_downcast_mut::<ImageAsset, _>(|asset| {
                assert!(in_band_bytes.is_empty());
                let name = asset.base.name().to_owned();
                let bytes = self
                    .images
                    .remove(&name)
                    .unwrap_or_else(|| panic!("unexpected out-of-band ImageAsset {name}"));
                asset.decode(&bytes, factory)
            })
            .unwrap_or(false)
    }
}

fn import(
    relative: &str,
    loader: Option<FileAssetLoaderRef>,
) -> (RuntimeFileHandle, PersistentFactory<RecordingFactory>) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&pinned_asset(relative), retained, None, loader, None)
        .unwrap_or_else(|| panic!("import {relative}"));
    (file, factory)
}

fn default_artboard(file: &RuntimeFileHandle) -> RuntimeArtboardInstanceHandle {
    file.with_file(|file| file.artboard_default())
        .expect("default artboard")
}

fn image_asset_for_named_image(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    let image = artboard
        .with_artboard(|artboard| artboard.find_handle::<Image>(name))
        .unwrap_or_else(|| panic!("missing Image named {name}"));
    image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .unwrap_or_else(|| panic!("Image {name} has no resolved ImageAsset"))
}

fn decoded_byte_size(asset: &CoreHandle) -> usize {
    asset
        .with_downcast::<ImageAsset, _>(|asset| asset.decoded_byte_size)
        .expect("actual ImageAsset owner")
}

fn assert_named_image_owners(artboard: &RuntimeArtboardInstanceHandle) -> (CoreHandle, CoreHandle) {
    let walle = image_asset_for_named_image(artboard, "walle");
    let eve_left = image_asset_for_named_image(artboard, "eve_left");
    let eve_right = image_asset_for_named_image(artboard, "eve_right");
    assert_ne!(eve_right, walle);
    assert_eq!(eve_right, eve_left);
    (walle, eve_left)
}

fn update_and_draw(
    artboard: &RuntimeArtboardInstanceHandle,
    factory: &mut PersistentFactory<RecordingFactory>,
) {
    let _ = Artboard::update_components_handle(&artboard.core_handle());
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    assert_eq!(factory.borrow().stream().matches("decodeImage ").count(), 2);
}

#[test]
fn wave_b5_image_assets_load_correctly() {
    let (file, mut factory) = import("walle.riv", None);
    let artboard = default_artboard(&file);
    let (walle, eve) = assert_named_image_owners(&artboard);
    assert_eq!(decoded_byte_size(&walle), 218_873);
    assert_eq!(decoded_byte_size(&eve), 246_825);
    update_and_draw(&artboard, &mut factory);
}

#[test]
fn wave_b5_out_of_band_image_assets_load_correctly() {
    let walle_bytes = pinned_asset("out_of_band/walle-370.png");
    let eve_bytes = pinned_asset("out_of_band/eve-317.png");
    assert_eq!(walle_bytes.len(), 218_873);
    assert_eq!(eve_bytes.len(), 246_825);
    let loader = FileAssetLoaderRef::new(Box::new(ExternalImageLoader {
        images: BTreeMap::from([
            ("walle.jpg".to_owned(), walle_bytes),
            ("eve.png".to_owned(), eve_bytes),
        ]),
    }));
    let (file, mut factory) = import("out_of_band/walle.riv", Some(loader));
    let artboard = default_artboard(&file);
    let (walle, eve) = assert_named_image_owners(&artboard);
    assert_eq!(decoded_byte_size(&walle), 218_873);
    assert_eq!(decoded_byte_size(&eve), 246_825);
    update_and_draw(&artboard, &mut factory);
}
