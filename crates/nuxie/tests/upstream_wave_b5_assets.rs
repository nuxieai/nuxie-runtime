//! Exact executable ports of pinned `image_asset_test.cpp`.

use std::path::PathBuf;

use nuxie::{File, RecordingFactory};

fn pinned_asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn image_asset_global_for_named_image(file: &File, name: &str) -> u32 {
    let image = file
        .default_artboard()
        .expect("default artboard")
        .graph()
        .component_named(name)
        .unwrap_or_else(|| panic!("missing Image named {name}"));
    assert_eq!(image.type_name, "Image");
    let object = file
        .runtime()
        .object(image.global_id as usize)
        .unwrap_or_else(|| panic!("missing runtime Image named {name}"));
    file.runtime()
        .resolved_file_asset_for_referencer(object)
        .unwrap_or_else(|| panic!("Image {name} has no resolved ImageAsset"))
        .id
}

fn image_asset_contents_len(file: &File, global_id: u32) -> Option<usize> {
    file.assets()
        .find(|asset| asset.descriptor().id == global_id)
        .and_then(|asset| asset.contents())
        .map(<[u8]>::len)
}

fn assert_named_image_owners(file: &File) -> (u32, u32) {
    let walle = image_asset_global_for_named_image(file, "walle");
    let eve_left = image_asset_global_for_named_image(file, "eve_left");
    let eve_right = image_asset_global_for_named_image(file, "eve_right");
    assert_ne!(eve_right, walle);
    assert_eq!(eve_right, eve_left);
    (walle, eve_left)
}

fn update_and_draw(file: &File) {
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate image artboard");
    artboard.raw_mut().update_components();
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    artboard
        .draw(&mut factory, &mut renderer)
        .expect("draw image artboard");
    assert_eq!(factory.stream().matches("decodeImage ").count(), 2);
}

#[test]
fn wave_b5_image_assets_load_correctly() {
    let file = File::import(&pinned_asset("walle.riv")).expect("import walle.riv");
    let (walle, eve) = assert_named_image_owners(&file);
    assert_eq!(image_asset_contents_len(&file, walle), Some(218_873));
    assert_eq!(image_asset_contents_len(&file, eve), Some(246_825));
    update_and_draw(&file);
}

#[test]
fn wave_b5_out_of_band_image_assets_load_correctly() {
    let mut file =
        File::import(&pinned_asset("out_of_band/walle.riv")).expect("import out-of-band walle.riv");
    let (walle, eve) = assert_named_image_owners(&file);
    assert_eq!(image_asset_contents_len(&file, walle), None);
    assert_eq!(image_asset_contents_len(&file, eve), None);

    let walle_bytes = pinned_asset("out_of_band/walle-370.png");
    let eve_bytes = pinned_asset("out_of_band/eve-317.png");
    assert_eq!(walle_bytes.len(), 218_873);
    assert_eq!(eve_bytes.len(), 246_825);
    for (asset_id, name) in file
        .assets()
        .map(|asset| {
            (
                asset.asset_id().expect("ImageAsset assetId"),
                asset.name().expect("ImageAsset name").to_owned(),
            )
        })
        .collect::<Vec<_>>()
    {
        let bytes = match name.as_str() {
            "walle.jpg" => walle_bytes.clone(),
            "eve.png" => eve_bytes.clone(),
            other => panic!("unexpected out-of-band ImageAsset {other}"),
        };
        file.attach_external_image_asset_bytes(asset_id, bytes)
            .expect("attach exact external ImageAsset bytes");
    }
    update_and_draw(&file);
}
