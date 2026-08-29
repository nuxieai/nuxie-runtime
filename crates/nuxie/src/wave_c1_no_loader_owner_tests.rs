use super::*;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = std::path::PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
#[ignore = "expected-red: no-loader File::import leaves the in-band ImageAsset undecoded instead of retaining the pinned 308-byte source payload"]
fn wave_c1_in_band_asset_001_no_loader_import_flow() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &pinned_fixture("in_band_asset.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("no-loader File::import succeeds");
    let assets = file.with_file(|file| file.assets().to_vec());
    assert_eq!(assets.len(), 1);
    let asset = assets[0].clone();
    asset
        .with(|object| {
            let image = object
                .as_any()
                .downcast_ref::<nuxie_runtime::source::assets::image_asset::ImageAsset>()
                .expect("ImageAsset");
            let file_asset = object.as_file_asset().expect("FileAsset").file_asset_base();
            assert_eq!(file_asset.cdn_uuid_str(), "");
            assert_eq!(
                file_asset.cdn_base_url(),
                "https://public.rive.app/cdn/uuid"
            );
            assert_eq!(
                file_asset.unique_filename(image.file_extension()),
                "1x1-45022.png"
            );
            assert_eq!(image.file_extension(), "png");
            assert_eq!(image.decoded_byte_size, 308);
            assert!(
                image.render_image().is_some(),
                "no-loader import must decode and retain the in-band ImageAsset"
            );
        })
        .expect("live ImageAsset");
}
