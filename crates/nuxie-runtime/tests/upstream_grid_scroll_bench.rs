//! Literal hidden benchmark from grid_scroll_bench_test.cpp at 160085c6.
//! Kept ignored like upstream's [.bench]; normal validation is not a performance campaign.
#[path = "support/virtual_scroll.rs"]
mod support;
use nuxie_runtime::source::viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem;
use std::time::Instant;
use support::*;

fn atoi(value: &str) -> i32 {
    let mut chars = value.trim_start().chars().peekable();
    let negative = match chars.peek() {
        Some('-') => {
            chars.next();
            true
        }
        Some('+') => {
            chars.next();
            false
        }
        _ => false,
    };
    let mut value = 0i32;
    for c in chars {
        let Some(digit) = c.to_digit(10).filter(|_| c.is_ascii()) else {
            break;
        };
        value = value.wrapping_mul(10).wrapping_add(digit as i32);
    }
    if negative {
        value.wrapping_neg()
    } else {
        value
    }
}
fn ms(from: Instant) -> f64 {
    from.elapsed().as_secs_f64() * 1000.0
}

#[test]
#[ignore = "upstream hidden [.bench]; run only for explicitly requested performance work"]
fn bench_virtualized_grid_scroll() {
    let max_count = std::env::var("BENCH_MAX")
        .map(|s| atoi(&s))
        .unwrap_or(100_000);
    let counts = std::env::var("BENCH_REPEAT")
        .map(|s| vec![100_000; usize::try_from(atoi(&s)).expect("nonnegative repeat count")])
        .unwrap_or_else(|_| vec![1_000, 10_000, 100_000]);
    for count in counts {
        if count > max_count {
            break;
        }
        let file = read_file("layout/layout_scroll_grid_virtualized.riv");
        let artboard = file.with_file(|f| f.artboard_named("Main")).unwrap();
        if std::env::var_os("BENCH_NOVIRT").is_some() {
            let scroll =
                artboard.with_artboard(|a| a.find_all_handles::<ScrollConstraint>()[0].clone());
            boolean(&scroll, Scroll::VIRTUALIZE_PROPERTY_KEY, false);
        }
        let vmi = file
            .with_file_mut(|f| {
                f.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .unwrap();
        let sm = if artboard.with_artboard(|a| a.state_machine_count()) > 0 {
            artboard.default_state_machine()
        } else {
            None
        };
        if let Some(sm) = &sm {
            sm.with_instance_mut(|s| s.bind_view_model_instance(vmi.clone()));
        } else {
            artboard.bind_view_model_instance(Some(vmi.clone()));
        }
        let items = read::<ViewModelInstance, _>(&vmi, |v| {
            v.property_values()
                .iter()
                .filter(|v| v.is_type_of(ViewModelInstanceList::TYPE_KEY))
                .last()
                .cloned()
        })
        .unwrap();
        let first = read::<ViewModelInstanceList, _>(&items, |v| v.list_items()[0].clone());
        let first_vmi =
            read::<ViewModelInstanceListItem, _>(&first, |v| v.view_model_instance()).unwrap();
        let model = read::<ViewModelInstance, _>(&first_vmi, |v| v.get_view_model()).unwrap();
        let make_item = || {
            let item = items
                .insert_sibling(ViewModelInstanceListItem::default())
                .unwrap();
            let instance = file
                .with_file_mut(|f| f.create_view_model_instance(model.clone()))
                .unwrap();
            write::<ViewModelInstanceListItem, _>(&item, |i| {
                i.set_view_model_instance(Some(instance));
                i.set_artboard(read::<ViewModelInstanceListItem, _>(&first, |i| {
                    i.artboard()
                }));
            });
            item
        };
        let setup_start = Instant::now();
        while read::<ViewModelInstanceList, _>(&items, |v| v.list_items().len() as i32) < count {
            let item = make_item();
            write::<ViewModelInstanceList, _>(&items, |v| {
                if v.list_items().len() as i32 + 1 < count {
                    v.internal_add_item(item);
                } else {
                    v.add_item(item);
                }
            });
        }
        let create = ms(setup_start);
        let frame = |seconds| {
            if let Some(sm) = &sm {
                sm.advance_and_apply(seconds);
            } else {
                artboard.advance_default(seconds);
            }
        };
        let scroll =
            artboard.with_artboard(|a| a.find_all_handles::<ScrollConstraint>()[0].clone());
        let mut first_frame = 0.0;
        for i in 0..3 {
            let frame_start = Instant::now();
            frame(0.0);
            if i == 0 {
                first_frame = ms(frame_start);
            }
        }
        let settle = ms(setup_start);
        println!(
            "LOAD items {count:6}  create+add {create:8.2} ms  first frame {first_frame:8.2} ms  next two {:8.2} ms",
            settle - create - first_frame
        );
        if std::env::var_os("BENCH_LOAD_ONLY").is_some() {
            continue;
        }
        let frames = 600;
        let mut worst = 0.0f64;
        let start = Instant::now();
        for f in 0..frames {
            let frame_start = Instant::now();
            number(
                &scroll,
                Scroll::SCROLL_OFFSET_Y_PROPERTY_KEY,
                -(f as f32) * 25.0,
            );
            frame(0.016);
            worst = worst.max(ms(frame_start));
        }
        let scrolling = ms(start) / frames as f64;
        let start = Instant::now();
        for _ in 0..frames {
            frame(0.016);
        }
        let idle = ms(start) / frames as f64;
        let extra = make_item();
        let start = Instant::now();
        write::<ViewModelInstanceList, _>(&items, |v| v.add_item(extra));
        frame(0.016);
        let append = ms(start);
        println!("APPEND items {count:6}  add one item + frame {append:9.2} ms");
        println!(
            "BENCH {} items {count:6}  load {settle:9.2} ms  scroll {scrolling:8.3} ms/frame (worst {worst:8.3})  idle {idle:8.3} ms/frame  content {:.0}",
            if sm.is_some() { "sm" } else { "ab" },
            read::<ScrollConstraint, _>(&scroll, |s| s.content_height())
        );
    }
}
