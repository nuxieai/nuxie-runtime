//! Direct translation of component_list_grid_virtualized_test.cpp at 160085c6.
#[path = "support/virtual_scroll.rs"]
mod support;
use nuxie_runtime::source::{
    generated::layout::grid_track_base::GridTrackBase,
    layout::grid_track::{GridTrack, GridTrackCollection},
    viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem,
};
use support::*;
#[test]
fn grid_virtualized_list_windows_rows() {
    let f = ScrollFixture::grid();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtualizes_grid()));
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtual_axis_is_column()));
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        360.0,
    );
    assert_eq!(f.realized(), (0..12).collect::<Vec<_>>());
}
#[test]
fn grid_virtualized_list_places_items_in_cells() {
    let f = ScrollFixture::grid();
    let b = f.bounds(1);
    approx(b.left(), 110.0);
    approx(b.top(), 10.0);
    let b = f.bounds(19);
    approx(b.left(), 330.0);
    approx(b.top(), 300.0);
    let origin = f.drawn_at(0);
    let at1 = f.drawn_at(1) - origin;
    approx(at1.x, 110.0);
    approx(at1.y, 0.0);
    let at5 = f.drawn_at(5) - origin;
    approx(at5.x, 110.0);
    approx(at5.y, 80.0);
}
#[test]
fn grid_virtualized_list_recycles_whole_rows() {
    let f = ScrollFixture::grid();
    f.offset_y(-100.0);
    f.settle();
    assert_eq!(f.realized(), (4..16).collect::<Vec<_>>());
    let origin = f.drawn_at(4);
    approx((f.drawn_at(6) - origin).x, 220.0);
    approx((f.drawn_at(8) - origin).y, 70.0);
}
#[test]
fn grid_virtualization_needs_vertical_scroll() {
    let f = ScrollFixture::grid();
    f.direction(0);
    assert!(!read::<ScrollConstraint, _>(&f.scroll, |s| s.virtualizes_grid()));
}
#[test]
fn vertical_grid_windows_columns_it_cannot_show() {
    let f = ScrollFixture::grid();
    f.narrow_grid();
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtualizes_grid_columns()));
    assert!(!read::<ScrollConstraint, _>(&f.scroll, |s| s.indexes_grid_cells()));
    assert_eq!(f.realized(), vec![0, 1, 2, 4, 5, 6, 8, 9, 10]);
    let origin = f.drawn_at(0);
    approx((f.drawn_at(2) - origin).x, 220.0);
    approx((f.drawn_at(4) - origin).y, 80.0);
    f.scroll_y(-100.0);
    f.settle();
    assert_eq!(f.realized(), vec![4, 5, 6, 8, 9, 10, 12, 13, 14]);
    f.scroll_y(-45.0);
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.scroll_index()),
        0.5,
    );
}
#[test]
fn grid_columns_window_from_content_position() {
    let f = ScrollFixture::grid();
    f.narrow_grid();
    let viewport = read::<ScrollConstraint, _>(&f.scroll, |s| s.viewport_handle()).unwrap();
    uint(
        &style(&viewport),
        Style::LAYOUT_ALIGNMENT_TYPE_PROPERTY_KEY,
        4,
    );
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtualizes_grid_columns()));
    approx(
        read::<LayoutComponent, _>(&f.content(), |c| c.layout_x()),
        -90.0,
    );
    assert!(f.realized().contains(&0));
    assert!(f.realized().contains(&3));
    approx((f.drawn_at(3) - f.drawn_at(0)).x, 330.0);
}
#[test]
fn grid_virtualization_off_unpins_cells() {
    let f = ScrollFixture::grid();
    f.offset_y(-100.0);
    f.settle();
    assert_eq!(f.realized(), (4..16).collect::<Vec<_>>());
    boolean(&f.scroll, Scroll::VIRTUALIZE_PROPERTY_KEY, false);
    f.settle();
    let origin = f.drawn_at(0);
    let at4 = f.drawn_at(4) - origin;
    approx(at4.x, 0.0);
    approx(at4.y, 80.0);
    let at19 = f.drawn_at(19) - origin;
    approx(at19.x, 330.0);
    approx(at19.y, 290.0);
}
#[test]
fn grid_both_ways_windows_rows_and_columns() {
    let f = ScrollFixture::grid();
    f.narrow_grid();
    f.direction(2);
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.virtualizes_grid_columns()));
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_width()),
        430.0,
    );
    assert_eq!(f.realized(), vec![0, 1, 2, 4, 5, 6, 8, 9, 10]);
    f.offset_x(-150.0);
    f.settle();
    assert_eq!(f.realized(), vec![1, 2, 3, 5, 6, 7, 9, 10, 11]);
    let origin = f.drawn_at(1);
    approx((f.drawn_at(3) - origin).x, 220.0);
    approx((f.drawn_at(5) - origin).y, 80.0);
    let x = read::<LayoutComponent, _>(&f.content(), |c| c.world_transform()[4]);
    approx(origin.x - x, 110.0 - 150.0);
}
#[test]
fn grid_scrolled_across_anchors_to_item_on_screen() {
    let f = ScrollFixture::grid();
    f.narrow_grid();
    f.direction(2);
    f.settle();
    f.scroll_x(-150.0);
    f.scroll_y(-20.0);
    f.settle();
    assert_eq!(f.realized(), vec![1, 2, 3, 5, 6, 7, 9, 10, 11]);
    let items = read::<ViewModelInstance, _>(&f.instance, |v| {
        v.property_values()
            .iter()
            .find(|v| v.is_type_of(ViewModelInstanceList::TYPE_KEY))
            .cloned()
    })
    .unwrap();
    write::<ViewModelInstanceList, _>(&items, |items| items.swap(1, 5));
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_y()),
        -110.0,
    );
}
#[test]
fn grid_carousel_both_ways_cycles_rows_and_columns() {
    let f = ScrollFixture::grid();
    f.narrow_grid();
    f.direction(2);
    boolean(&f.scroll, Scroll::INFINITE_PROPERTY_KEY, true);
    f.settle();
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.loops_x()));
    assert!(read::<ScrollConstraint, _>(&f.scroll, |s| s.loops_y()));
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_width()),
        440.0,
    );
    assert_eq!(f.realized(), vec![0, 1, 2, 4, 5, 6, 8, 9, 10]);
    f.scroll_x(-330.0);
    f.settle();
    assert_eq!(f.realized(), vec![0, 1, 3, 4, 5, 7, 8, 9, 11]);
    assert_eq!(
        write::<ArtboardComponentList, _>(&f.list, |list| list.ordered_list_indices().to_vec()),
        vec![3, 0, 1, 7, 4, 5, 11, 8, 9]
    );
    approx((f.drawn_at(0) - f.drawn_at(3)).x, 110.0);
    approx((f.drawn_at(1) - f.drawn_at(0)).x, 110.0);
    approx((f.drawn_at(4) - f.drawn_at(0)).y, 80.0);
    f.scroll_y(-460.0);
    f.settle();
    assert_eq!(f.realized(), vec![4, 5, 7, 8, 9, 11, 12, 13, 15]);
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.scroll_index()),
        7.0,
    );
}
#[test]
fn grid_auto_columns_size_from_unrealized_items() {
    let f = ScrollFixture::grid();
    let tracks = f
        .artboard
        .with_artboard(|a| a.find_all_handles::<GridTrack>());
    for track in tracks {
        if read::<GridTrack, _>(&track, |t| t.grid_collection())
            == GridTrackCollection::TemplateColumns
        {
            uint(&track, GridTrackBase::TRACK_TYPE_PROPERTY_KEY, 0); // GridTrackSizeType::autoSize
            break;
        }
    }
    write::<ArtboardComponentList, _>(&f.list, |list| {
        list.set_item_size(Vec2D::new(150.0, 60.0), 16)
    });
    f.settle();
    approx(f.bounds(1).left(), 160.0);
    approx((f.drawn_at(1) - f.drawn_at(0)).x, 160.0);
}

#[test]
fn grid_virtualized_list_keeps_its_offset_while_layout_places_rows() {
    let f = ScrollFixture::grid_settled(false);
    f.scroll_y(-100.0);
    f.offset_y(-100.0);
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.offset_y()),
        -100.0,
    );
}

#[test]
fn grid_virtualized_list_with_percent_rows_scrolls_its_layout_height() {
    let f = ScrollFixture::grid();
    let content = f.content();
    uint(
        &style(&content),
        Sizing::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
        2,
    );
    let children = read::<LayoutComponent, _>(&content, |c| c.base.children().to_vec());
    for child in children {
        if child.is_type_of(GridTrack::TYPE_KEY)
            && read::<GridTrack, _>(&child, |t| t.grid_collection())
                == GridTrackCollection::TemplateRows
        {
            uint(&child, GridTrackBase::TRACK_TYPE_PROPERTY_KEY, 2);
            number(&child, GridTrackBase::TRACK_VALUE_PROPERTY_KEY, 50.0);
        }
    }
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        read::<LayoutComponent, _>(&content, |c| c.layout_height()),
    );
}

#[test]
fn grid_virtualized_list_rebuilds_when_an_added_item_opens_a_row() {
    let f = ScrollFixture::grid();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        360.0,
    );
    let items = read::<ViewModelInstance, _>(&f.instance, |v| {
        v.property_values()
            .iter()
            .filter(|v| v.is_type_of(ViewModelInstanceList::TYPE_KEY))
            .last()
            .cloned()
    })
    .expect("list property");
    let first = read::<ViewModelInstanceList, _>(&items, |v| v.list_items()[0].clone());
    let first_instance =
        read::<ViewModelInstanceListItem, _>(&first, |i| i.view_model_instance()).unwrap();
    let model = read::<ViewModelInstance, _>(&first_instance, |v| v.get_view_model()).unwrap();
    let instance = f
        .file
        .with_file_mut(|f| f.create_view_model_instance(model))
        .unwrap();
    let item = items
        .insert_sibling(ViewModelInstanceListItem::default())
        .unwrap();
    write::<ViewModelInstanceListItem, _>(&item, |i| {
        i.set_view_model_instance(Some(instance));
        i.set_artboard(read::<ViewModelInstanceListItem, _>(&first, |i| {
            i.artboard()
        }));
    });
    write::<ViewModelInstanceList, _>(&items, |v| v.add_item(item));
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        430.0,
    );
}

#[test]
fn grid_virtualized_list_rebuilds_when_a_wider_gap_spreads_the_rows() {
    let f = ScrollFixture::grid();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        360.0,
    );
    number(&f.content_style(), Style::GAP_VERTICAL_PROPERTY_KEY, 20.0);
    f.settle();
    approx(
        read::<ScrollConstraint, _>(&f.scroll, |s| s.content_height()),
        400.0,
    );
}
