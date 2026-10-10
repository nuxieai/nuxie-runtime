//! Literal translation of viewmodel_instance_viewmodel_import_test.cpp at 439ffe0c.
#[path = "support/riv_bytes.rs"]
mod riv_bytes;
#[path = "support/import_439.rs"]
mod support;
use nuxie_runtime::source::generated::viewmodel::viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase;
use riv_bytes::{RivBytes, write_artboard};
#[test]
fn a_view_model_instance_view_model_with_no_instance_open_fails_its_import() {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    riv.object(ViewModelInstanceViewModelBase::TYPE_KEY);
    riv.end();
    let file = support::import(&riv.bytes());
    let objects = support::objects(&file);
    assert_eq!(objects.len(), 2);
    assert!(objects[1].is_none());
}
