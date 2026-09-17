use nuxie::{FileImportLimits, import_native};
use nuxie_binary::{FixtureProperty as P, FixtureRecord as R, FixtureValue as V};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    file_asset_loader::{FileAssetLoader, FileAssetLoaderRef},
};
use std::{cell::Cell, rc::Rc};

fn scene(embedded: bool) -> Vec<u8> {
    let mut records = vec![
        R {
            type_key: 23,
            properties: vec![],
        },
        R {
            type_key: 60000,
            properties: vec![P {
                key: 60000,
                value: V::String("video.mp4".into()),
            }],
        },
    ];
    if embedded {
        records.push(R {
            type_key: 106,
            properties: vec![P {
                key: 212,
                value: V::Bytes(vec![1, 2, 3, 4]),
            }],
        });
    }
    records.push(R {
        type_key: 1,
        properties: vec![],
    });
    records.push(R {
        type_key: 60001,
        properties: vec![
            P {
                key: 5,
                value: V::Uint(0),
            },
            P {
                key: 206,
                value: V::Uint(0),
            },
        ],
    });
    nuxie_binary::encode_runtime_file(
        &nuxie_binary::RuntimeFile::from_fixture_records(records).unwrap(),
    )
    .unwrap()
}
struct Loader(Rc<Cell<usize>>);
impl FileAssetLoader for Loader {
    fn load_contents(&mut self, _: CoreHandle, _: &[u8], _: &RuntimeFactoryHandle) -> bool {
        self.0.set(self.0.get() + 1);
        false
    }
}
#[test]
fn unavailable_video_is_rejected_before_any_loader_even_with_unbounded_allocations() {
    for limits in [FileImportLimits::default(), FileImportLimits::unbounded()] {
        for embedded in [false, true] {
            let count = Rc::new(Cell::new(0));
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let result = import_native(
                &scene(embedded),
                &mut factory,
                Some(FileAssetLoaderRef::new(Box::new(Loader(count.clone())))),
                limits,
            );
            assert!(result.is_err());
            assert!(
                result
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("video playback capability")
            );
            assert_eq!(count.get(), 0);
            let enabled = import_native(
                &scene(embedded),
                &mut factory,
                Some(FileAssetLoaderRef::new(Box::new(Loader(count.clone())))),
                limits.with_video_playback(true),
            );
            assert!(enabled.is_ok());
            assert!(count.get() > 0, "positive control must reach the loader");
        }
    }
}
#[test]
fn explicit_video_capability_preserves_per_asset_embedded_limits() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let enabled = FileImportLimits::default().with_video_playback(true);
    assert!(
        import_native(
            &scene(false),
            &mut factory,
            None,
            enabled.with_max_embedded_video_bytes(0)
        )
        .is_ok()
    );
    assert!(
        import_native(
            &scene(true),
            &mut factory,
            None,
            enabled.with_max_embedded_video_bytes(4)
        )
        .is_ok()
    );
    let result = import_native(
        &scene(true),
        &mut factory,
        None,
        enabled.with_max_embedded_video_bytes(3),
    );
    assert!(result.is_err());
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("embedded video exceeds")
    );
}
