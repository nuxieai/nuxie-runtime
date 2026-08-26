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
    let file = File::import(&pinned_fixture("in_band_asset.riv"))
        .expect("no-loader File::import succeeds");
    let assets = file.runtime().file_assets();
    assert_eq!(assets.len(), 1);
    let asset = assets[0];
    assert_eq!(asset.type_name, "ImageAsset");
    assert_eq!(asset.file_asset_cdn_uuid_string().as_deref(), Some(""));
    assert_eq!(
        asset.string_property("cdnBaseUrl"),
        Some("https://public.rive.app/cdn/uuid"),
    );
    assert_eq!(
        asset.file_asset_unique_filename().as_deref(),
        Some("1x1-45022.png"),
    );
    assert_eq!(asset.file_asset_extension(), Some("png"));
    assert_eq!(
        file.runtime()
            .imported_file_asset_contents(asset.id)
            .expect("in-band source payload")
            .len(),
        308,
    );
    assert!(
        file.file_asset_owners
            .image_assets()
            .get(asset.id)
            .is_some(),
        "no-loader import must decode and retain the in-band ImageAsset"
    );
}
