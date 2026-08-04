//! Observation coverage for solved `LayoutComponent` border boxes.

use anyhow::Result;
use nuxie::{
    Aabb, ArtboardId, ArtboardSpec, LayoutComponentSpec, LayoutComponentStyleSpec, NodeSpec,
    Parent, ResolveError, Scene, SceneLayoutFlexDirection, SceneLayoutScale,
};

fn layout(
    name: &str,
    width: f32,
    height: f32,
    width_scale: SceneLayoutScale,
    height_scale: SceneLayoutScale,
) -> NodeSpec {
    NodeSpec::LayoutComponent(LayoutComponentSpec {
        name: name.into(),
        x: 0.0,
        y: 0.0,
        opacity: 1.0,
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        clip: false,
        width,
        height,
        fractional_width: 1.0,
        fractional_height: 1.0,
        style: LayoutComponentStyleSpec {
            layout_width_scale: width_scale,
            layout_height_scale: height_scale,
            ..LayoutComponentStyleSpec::default()
        },
    })
}

fn settle(scene: &mut Scene, artboard: ArtboardId) -> Result<nuxie::InstanceId> {
    let instance = scene.instantiate(artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    Ok(instance)
}

#[test]
fn fill_container_reports_solved_border_box_instead_of_content_union() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, container), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Phone".into(),
            width: 393.0,
            height: 852.0,
        })?;
        let container = tx.create(
            Parent::Artboard(artboard),
            layout(
                "Fill container",
                0.0,
                0.0,
                SceneLayoutScale::Fill,
                SceneLayoutScale::Fill,
            ),
        )?;
        tx.create(
            Parent::Object(container),
            layout(
                "Content",
                393.0,
                238.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        Ok((artboard, container))
    })?;
    let instance = settle(&mut scene, artboard)?;

    assert_eq!(
        scene.solved_layout_bounds(instance, container)?,
        Aabb::new(0.0, 0.0, 393.0, 852.0),
        "the border box is the fill solve, not the 393x238 content union",
    );
    Ok(())
}

#[test]
fn hug_container_reports_its_hugged_solve() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, container), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Hug".into(),
            width: 500.0,
            height: 852.0,
        })?;
        // The style-less artboard root flows as a column (upstream Yoga
        // zero-init default), so author an explicit Row wrapper to keep the
        // horizontal-spacer offset this test exercises.
        let row = tx.create(
            Parent::Artboard(artboard),
            layout(
                "Row wrapper",
                500.0,
                852.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        tx.create(
            Parent::Object(row),
            layout(
                "Horizontal spacer",
                17.0,
                852.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        let mut column_node = layout(
            "Column",
            393.0,
            852.0,
            SceneLayoutScale::Fixed,
            SceneLayoutScale::Fixed,
        );
        let NodeSpec::LayoutComponent(column_spec) = &mut column_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        column_spec.style.flex_direction = SceneLayoutFlexDirection::Column;
        let outer = tx.create(Parent::Object(row), column_node)?;
        tx.create(
            Parent::Object(outer),
            layout(
                "Vertical spacer",
                393.0,
                19.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        let container = tx.create(
            Parent::Object(outer),
            layout(
                "Hug container",
                0.0,
                0.0,
                SceneLayoutScale::Hug,
                SceneLayoutScale::Hug,
            ),
        )?;
        tx.create(
            Parent::Object(container),
            layout(
                "Content",
                123.0,
                45.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        Ok((artboard, container))
    })?;
    let instance = settle(&mut scene, artboard)?;

    assert_eq!(
        scene.solved_layout_bounds(instance, container)?,
        Aabb::new(17.0, 19.0, 140.0, 64.0),
        "nested observations retain the artboard-space offset as well as the hugged size",
    );
    Ok(())
}

#[test]
fn solved_layout_bounds_distinguishes_foreign_and_unknown_nodes() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, node_a, artboard_b, node_b), _) = scene.edit(|tx| {
        let artboard_a = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "A".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let node_a = tx.create(
            Parent::Artboard(artboard_a),
            layout(
                "A node",
                10.0,
                10.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        let artboard_b = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "B".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let node_b = tx.create(
            Parent::Artboard(artboard_b),
            layout(
                "B node",
                10.0,
                10.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        Ok((artboard_a, node_a, artboard_b, node_b))
    })?;
    let instance_a = settle(&mut scene, artboard_a)?;
    let _ = (node_a, artboard_b);

    assert_eq!(
        scene.solved_layout_bounds(instance_a, node_b).unwrap_err(),
        ResolveError::DifferentArtboard,
    );

    let mut foreign = Scene::new();
    let ((foreign_artboard, foreign_node), _) = foreign.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Foreign".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let node = tx.create(
            Parent::Artboard(artboard),
            layout(
                "Foreign node",
                10.0,
                10.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        Ok((artboard, node))
    })?;
    let _ = foreign_artboard;
    assert_eq!(
        scene
            .solved_layout_bounds(instance_a, foreign_node)
            .unwrap_err(),
        ResolveError::UnknownObject,
    );
    Ok(())
}

#[test]
fn artboard_layout_style_default_row_flows_top_level_children_horizontally() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, first, second), _) = scene.edit(|tx| {
        // An empty style spec matches the upstream editor default (Row):
        // authoring ANY root style flips the artboard off the style-less
        // Column fallback, exactly like a Rive-editor-authored artboard.
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: Some(LayoutComponentStyleSpec::default()),
            name: "Row root".into(),
            width: 300.0,
            height: 100.0,
        })?;
        let first = tx.create(
            Parent::Artboard(artboard),
            layout(
                "First",
                100.0,
                20.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        let second = tx.create(
            Parent::Artboard(artboard),
            layout(
                "Second",
                100.0,
                20.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        Ok((artboard, first, second))
    })?;
    let instance = settle(&mut scene, artboard)?;

    assert_eq!(
        scene.solved_layout_bounds(instance, first)?,
        Aabb::new(0.0, 0.0, 100.0, 20.0),
    );
    assert_eq!(
        scene.solved_layout_bounds(instance, second)?,
        Aabb::new(100.0, 0.0, 200.0, 20.0),
        "a Row root style flows top-level children on the x axis",
    );
    Ok(())
}

#[test]
fn artboard_layout_style_column_gap_and_padding_reach_the_root() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, first, second), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: Some(LayoutComponentStyleSpec {
                flex_direction: SceneLayoutFlexDirection::Column,
                gap_vertical: 8.0,
                padding_left: 3.0,
                padding_top: 5.0,
                ..LayoutComponentStyleSpec::default()
            }),
            name: "Column root".into(),
            width: 300.0,
            height: 100.0,
        })?;
        let first = tx.create(
            Parent::Artboard(artboard),
            layout(
                "First",
                100.0,
                20.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        let second = tx.create(
            Parent::Artboard(artboard),
            layout(
                "Second",
                100.0,
                20.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        Ok((artboard, first, second))
    })?;
    let instance = settle(&mut scene, artboard)?;

    assert_eq!(
        scene.solved_layout_bounds(instance, first)?,
        Aabb::new(3.0, 5.0, 103.0, 25.0),
        "root padding offsets the first child",
    );
    assert_eq!(
        scene.solved_layout_bounds(instance, second)?,
        Aabb::new(3.0, 33.0, 103.0, 53.0),
        "root gap separates the flowed children",
    );
    Ok(())
}

#[test]
fn set_artboard_layout_style_flip_resettles_the_root_flow() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, second), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: Some(LayoutComponentStyleSpec::default()),
            name: "Flip".into(),
            width: 300.0,
            height: 100.0,
        })?;
        tx.create(
            Parent::Artboard(artboard),
            layout(
                "First",
                100.0,
                20.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        let second = tx.create(
            Parent::Artboard(artboard),
            layout(
                "Second",
                100.0,
                20.0,
                SceneLayoutScale::Fixed,
                SceneLayoutScale::Fixed,
            ),
        )?;
        Ok((artboard, second))
    })?;
    let instance = settle(&mut scene, artboard)?;
    assert_eq!(
        scene.solved_layout_bounds(instance, second)?,
        Aabb::new(100.0, 0.0, 200.0, 20.0),
    );

    scene.edit(|tx| {
        tx.set_artboard(
            artboard,
            ArtboardSpec {
                layout_style: Some(LayoutComponentStyleSpec {
                    flex_direction: SceneLayoutFlexDirection::Column,
                    ..LayoutComponentStyleSpec::default()
                }),
                name: "Flip".into(),
                width: 300.0,
                height: 100.0,
            },
        )
    })?;
    let flipped = settle(&mut scene, artboard)?;
    assert_eq!(
        scene.solved_layout_bounds(flipped, second)?,
        Aabb::new(0.0, 20.0, 100.0, 40.0),
        "replacing the artboard spec re-settles the root flow as a column",
    );
    Ok(())
}
