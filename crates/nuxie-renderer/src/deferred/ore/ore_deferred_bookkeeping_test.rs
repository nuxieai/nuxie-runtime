//! Upstream tests/unit_tests/renderer/ore_deferred_bookkeeping_test.cpp at edcf7d9d.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    bind_group::validateSetBindGroup, gpu_resource::AnyResourceHandle,
    script_guards::kMaxDynamicOffsets,
};

#[test]
fn native_ubo_count_cannot_exceed_its_slice() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let layout = ctx
        .makeBindGroupLayout(&BindGroupLayoutDesc::default())
        .unwrap();
    for layout in [None, Some(&layout)] {
        let desc = BindGroupDesc {
            layout,
            ubos: &[],
            uboCount: 1,
            ..Default::default()
        };
        let mut error = String::new();
        assert!(!nuxie_ore_metal::bind_group_layout::validateBindGroupDesc(
            &desc,
            Some(&mut error)
        ));
        assert!(error.contains("uboCount 1 exceeds the supplied UBO entry count 0"));
        ctx.clearLastError();
        assert!(ctx.makeBindGroup(&desc).is_none());
        assert!(ctx.lastError().contains(&error));
    }
}
use nuxie_ore_metal::{context::ContextApi, types::*};

fn make_dynamic_group(
    ctx: &mut DeferredOreContext,
    binding_size: u32,
) -> (AnyResourceHandle, AnyResourceHandle, AnyResourceHandle) {
    let entries = [BindGroupLayoutEntry {
        binding: 0,
        kind: BindingKind::uniformBuffer,
        hasDynamicOffset: true,
        ..Default::default()
    }];
    let layout = ctx
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            entries: Some(&entries),
            entryCount: 1,
            ..Default::default()
        })
        .unwrap();
    let buffer = ctx
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::uniform,
            size: 1024,
            data: None,
            immutable: false,
            label: None,
        })
        .unwrap();
    let ubos = [UBOEntry {
        slot: 0,
        buffer: Some(&buffer),
        size: binding_size,
        ..Default::default()
    }];
    let group = ctx
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: 1,
            ..Default::default()
        })
        .unwrap();
    (group, layout, buffer)
}

#[test]
fn a_sizeless_dynamic_ubo_spans_the_rest_of_its_buffer() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let (whole, _layout, _buffer) = make_dynamic_group(&mut ctx, 0);
    let whole = whole.bindGroupBase().unwrap();
    assert_eq!(whole.dynamicRanges().len(), 1);
    assert_eq!(whole.dynamicRanges()[0].size, 1024);
    let mut err = String::new();
    assert!(validateSetBindGroup(
        0,
        Some(whole),
        Some(&[0]),
        1,
        256,
        Some(&mut err)
    ));
    assert!(!validateSetBindGroup(
        0,
        Some(whole),
        Some(&[256]),
        1,
        256,
        Some(&mut err)
    ));
    let (group, _layout, _buffer) = make_dynamic_group(&mut ctx, 256);
    let group = group.bindGroupBase().unwrap();
    assert_eq!(group.dynamicRanges().len(), 1);
    assert_eq!(group.dynamicRanges()[0].size, 256);
    assert_eq!(group.dynamicRanges()[0].bufferSize, 1024);
}

#[test]
fn set_bind_group_validation_catches_misuse() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let (group, _layout, _buffer) = make_dynamic_group(&mut ctx, 256);
    let group = group.bindGroupBase().unwrap();
    let mut err = String::new();
    assert!(validateSetBindGroup(
        0,
        Some(group),
        Some(&[768]),
        1,
        256,
        Some(&mut err)
    ));
    assert!(!validateSetBindGroup(
        0,
        Some(group),
        None,
        0,
        256,
        Some(&mut err)
    ));
    assert!(err.contains("count 0 does not match"));
    assert!(!validateSetBindGroup(
        0,
        Some(group),
        Some(&[100]),
        1,
        256,
        Some(&mut err)
    ));
    assert!(err.contains("multiple of 256"));
    assert!(!validateSetBindGroup(
        0,
        Some(group),
        Some(&[1024]),
        1,
        256,
        Some(&mut err)
    ));
    assert!(err.contains("past the end of its 1024 byte buffer"));
    assert!(!validateSetBindGroup(
        kMaxBindGroups,
        Some(group),
        Some(&[768]),
        1,
        256,
        Some(&mut err)
    ));
    assert!(err.contains("groupIndex"));
    assert!(!validateSetBindGroup(
        1,
        Some(group),
        Some(&[768]),
        1,
        256,
        Some(&mut err)
    ));
    assert!(err.contains("made for group 0, not 1"));
    let many = [0; kMaxDynamicOffsets as usize + 1];
    assert!(!validateSetBindGroup(
        0,
        Some(group),
        Some(&many),
        kMaxDynamicOffsets + 1,
        256,
        Some(&mut err)
    ));
    assert!(err.contains("exceeds maximum"));
}

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
    assert!(
        ctx.makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: 1,
            ..Default::default()
        })
        .is_none()
    );
    assert!(ctx.lastError().contains("needs 192"));

    buffer_desc.size = 192;
    let sized = ctx.makeBuffer(&buffer_desc).unwrap();
    ubos[0].buffer = Some(&sized);
    assert!(
        ctx.makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: 1,
            ..Default::default()
        })
        .is_some()
    );
}
