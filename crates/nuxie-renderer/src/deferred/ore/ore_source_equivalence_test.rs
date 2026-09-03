//! Regression witnesses for e949498e descriptor wire/replay contracts.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    ore_cmd::{
        ore_command_buffer::{OreCommandBuffer, OreCommandReader},
        ore_commands::{CommandType, MakeResourcePOD},
        ore_handle::INVALID_HANDLE,
        ore_make_recording::{
            recordMakeBindGroupLayout, recordMakePipeline, recordMakeShaderModule,
        },
        ore_make_replay::{decodePods, OreResident},
        ore_replay::replayOreStream,
        ore_resource_commands::*,
    },
    types::*,
};

#[test]
fn replay_preserves_the_complete_explicitly_sized_hlsl_source() {
    let source = "before\0after\0";
    let mut commands = OreCommandBuffer::default();
    recordMakeShaderModule(
        &mut commands,
        0,
        0,
        &ShaderModuleDesc {
            hlslSource: Some(source),
            hlslSourceSize: source.len() as u32,
            ..Default::default()
        },
    );
    let mut recorder = DeferredOreContext::fromReal(None);
    let mut residents = OreResident::default();
    replayOreStream(
        &mut recorder,
        commands.command_bytes(),
        commands.blob_bytes(),
        &mut residents,
        &mut |_| None,
        &mut |_| None,
        &mut |_| None,
    );
    assert!(residents.objects[0].is_some());
    let stream = recorder.stream();
    let stream = stream.borrow();
    let mut reader = OreCommandReader::new(stream.command_bytes(), stream.blob_bytes());
    assert_eq!(reader.next(), Some(CommandType::makeShaderModule));
    reader.read::<MakeResourcePOD>();
    let pod: ShaderModuleDescPOD = reader.read();
    assert_eq!(pod.hlslSource.size, source.len() as u32);
    assert_eq!(
        reader.blob_at(pod.hlslSource.offset, pod.hlslSource.size),
        source.as_bytes()
    );
}

#[test]
fn nullable_zero_count_descriptor_spans_survive_record_and_replay() {
    let mut commands = OreCommandBuffer::default();
    for (id, entries) in [None, Some(&[][..])].into_iter().enumerate() {
        recordMakeBindGroupLayout(
            &mut commands,
            id as u32,
            0,
            &BindGroupLayoutDesc {
                entries,
                ..Default::default()
            },
        );
    }
    let buffers = [
        VertexBufferLayout {
            attributes: None,
            ..Default::default()
        },
        VertexBufferLayout {
            attributes: Some(&[]),
            ..Default::default()
        },
    ];
    recordMakePipeline(
        &mut commands,
        2,
        0,
        &PipelineDesc {
            vertexBuffers: Some(&buffers),
            vertexBufferCount: 2,
            ..Default::default()
        },
        INVALID_HANDLE,
        INVALID_HANDLE,
        &[],
    );
    let mut recorder = DeferredOreContext::fromReal(None);
    let mut residents = OreResident::default();
    replayOreStream(
        &mut recorder,
        commands.command_bytes(),
        commands.blob_bytes(),
        &mut residents,
        &mut |_| None,
        &mut |_| None,
        &mut |_| None,
    );
    assert!(residents.objects.iter().all(Option::is_some));
    let replayed = recorder.stream();
    let replayed = replayed.borrow();
    for (bytes, blobs) in [
        (commands.command_bytes(), commands.blob_bytes()),
        (replayed.command_bytes(), replayed.blob_bytes()),
    ] {
        let mut reader = OreCommandReader::new(bytes, blobs);
        for absent in [true, false] {
            assert_eq!(reader.next(), Some(CommandType::makeBindGroupLayout));
            reader.read::<MakeResourcePOD>();
            let pod: BindGroupLayoutDescPOD = reader.read();
            assert_eq!(pod.entryCount, 0);
            assert_eq!(pod.entries.size, if absent { NO_BLOB.size } else { 0 });
            assert_eq!(pod.entries.absent(), absent);
        }
        assert_eq!(reader.next(), Some(CommandType::makePipeline));
        reader.read::<MakeResourcePOD>();
        let pod: PipelineDescPOD = reader.read();
        let buffers = decodePods::<VertexBufferLayoutPOD>(
            reader.blob_at(pod.vertexBuffers.offset, pod.vertexBuffers.size),
            pod.vertexBufferCount,
        );
        assert_eq!(buffers.len(), 2);
        for (buffer, absent) in buffers.iter().zip([true, false]) {
            assert_eq!(buffer.attributeCount, 0);
            assert_eq!(
                buffer.attributes.size,
                if absent { NO_BLOB.size } else { 0 }
            );
            assert_eq!(buffer.attributes.absent(), absent);
        }
    }
}
