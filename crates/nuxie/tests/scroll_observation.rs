//! Public Scene occurrence fencing for read-only ScrollConstraint observation.

use anyhow::Result;
use nuxie::{ArtboardSpec, ResolveError, Scene};

#[test]
fn scene_scroll_observation_is_scoped_to_one_live_instance() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Observation host".into(),
            width: 100.0,
            height: 100.0,
        })?;
        Ok(artboard)
    })?;
    let instance = scene.instantiate(artboard)?;

    assert!(scene.scroll_constraint_occurrences(instance)?.is_empty());
    assert_eq!(scene.scroll_constraint_for_content(instance, 0)?, None);
    assert_eq!(scene.scroll_constraint_for_authored_id(instance, 0)?, None);
    assert_eq!(
        scene.scroll_constraint_for_content_authored_id(instance, 0)?,
        None
    );

    scene.drop_instance(instance);
    assert_eq!(
        scene.scroll_constraint_occurrences(instance).unwrap_err(),
        ResolveError::UnknownInstance
    );
    Ok(())
}
