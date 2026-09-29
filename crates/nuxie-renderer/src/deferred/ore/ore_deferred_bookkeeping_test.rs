//! Upstream tests/unit_tests/renderer/ore_deferred_bookkeeping_test.cpp at edcf7d9d.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{context::ContextApi, types::*};

#[test]
fn a_deferred_layout_answers_for_its_entries() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let entries = [
        BindGroupLayoutEntry {
            binding: 0,
            kind: BindingKind::uniformBuffer,
            hasDynamicOffset: true,
            ..Default::default()
        },
        BindGroupLayoutEntry {
            binding: 1,
            kind: BindingKind::uniformBuffer,
            ..Default::default()
        },
    ];
    let layout = ctx
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            groupIndex: 2,
            entries: Some(&entries),
            entryCount: 2,
            ..Default::default()
        })
        .unwrap();
    let layout = layout.bindGroupLayoutBase().unwrap();
    assert_eq!(layout.groupIndex(), 2);
    assert_eq!(layout.entries().len(), 2);
    assert!(layout.hasDynamicOffset(0));
    assert!(!layout.hasDynamicOffset(1));
    assert!(!layout.hasDynamicOffset(7));
}

#[test]
fn a_deferred_bind_group_counts_its_dynamic_offsets() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let entries = [
        BindGroupLayoutEntry {
            binding: 0,
            kind: BindingKind::uniformBuffer,
            hasDynamicOffset: true,
            ..Default::default()
        },
        BindGroupLayoutEntry {
            binding: 1,
            kind: BindingKind::uniformBuffer,
            ..Default::default()
        },
    ];
    let layout = ctx
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            groupIndex: 3,
            entries: Some(&entries),
            entryCount: 2,
            ..Default::default()
        })
        .unwrap();
    let ubos = [
        UBOEntry {
            slot: 0,
            ..Default::default()
        },
        UBOEntry {
            slot: 1,
            ..Default::default()
        },
    ];
    let group = ctx
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: 2,
            ..Default::default()
        })
        .unwrap();
    let group = group.bindGroupBase().unwrap();
    assert_eq!(group.dynamicOffsetCount(), 1);
    assert_eq!(group.groupIndex(), 3);
    assert_eq!(
        group.layout().unwrap().allocation_identity(),
        layout.allocation_identity()
    );
}

#[test]
fn an_empty_deferred_layout_keeps_no_entries() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let layout = ctx
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            groupIndex: 1,
            ..Default::default()
        })
        .unwrap();
    let layout_base = layout.bindGroupLayoutBase().unwrap();
    assert!(layout_base.entries().is_empty());
    assert_eq!(layout_base.groupIndex(), 1);

    let group = ctx
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(group.bindGroupBase().unwrap().dynamicOffsetCount(), 0);
}

#[test]
fn a_deferred_bind_group_rejects_a_ubo_shorter_than_its_block() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let entries = [BindGroupLayoutEntry {
        binding: 1,
        kind: BindingKind::uniformBuffer,
        minBindingSize: 192,
        ..Default::default()
    }];
    let layout = ctx
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            entries: Some(&entries),
            entryCount: 1,
            ..Default::default()
        })
        .unwrap();
    let mut buffer_desc = BufferDesc {
        usage: BufferUsage::uniform,
        size: 160,
        data: None,
        immutable: false,
        label: None,
    };
    let model = ctx.makeBuffer(&buffer_desc).unwrap();
    let mut ubos = [UBOEntry {
        slot: 1,
        buffer: Some(&model),
        ..Default::default()
    }];
    // Replay never talks to the script, so the refusal happens on record.
    ctx.clearLastError();
    assert!(ctx
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: 1,
            ..Default::default()
        })
        .is_none());
    assert!(ctx.lastError().contains("needs 192"));

    buffer_desc.size = 192;
    let sized = ctx.makeBuffer(&buffer_desc).unwrap();
    ubos[0].buffer = Some(&sized);
    assert!(ctx
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: 1,
            ..Default::default()
        })
        .is_some());
}
