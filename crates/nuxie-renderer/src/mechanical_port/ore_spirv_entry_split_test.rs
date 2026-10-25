//! Literal public ore_spirv_entry_split_test.cpp at e579560ae.
#![allow(non_snake_case, non_upper_case_globals)]
use super::ore_spirv_entry_split::*;
const kTwoPairs: &[u32] = &[
    0x07230203, 0x00010000, 0x0000001c, 0x00000064, 0x00000000, 0x00020011, 0x00000001, 0x0006000b,
    0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e, 0x00000000, 0x0003000e, 0x00000000, 0x00000001,
    0x0008000f, 0x00000000, 0x00000021, 0x5f676f66, 0x00007376, 0x0000001a, 0x0000001d, 0x0000001f,
    0x0008000f, 0x00000004, 0x0000003a, 0x5f676f66, 0x00007366, 0x00000033, 0x00000036, 0x00000039,
    0x0009000f, 0x00000000, 0x00000048, 0x74737564, 0x0073765f, 0x00000041, 0x00000044, 0x00000045,
    0x00000046, 0x0009000f, 0x00000004, 0x0000005d, 0x74737564, 0x0073665f, 0x00000055, 0x00000057,
    0x00000059, 0x0000005c, 0x00030010, 0x0000003a, 0x00000007, 0x00030010, 0x0000005d, 0x00000007,
    0x00050048, 0x00000007, 0x00000000, 0x00000023, 0x00000000, 0x00050048, 0x00000007, 0x00000001,
    0x00000023, 0x00000010, 0x00050048, 0x00000009, 0x00000000, 0x00000023, 0x00000000, 0x00050048,
    0x00000009, 0x00000001, 0x00000023, 0x00000010, 0x00050048, 0x00000009, 0x00000002, 0x00000023,
    0x00000020, 0x00040047, 0x0000000a, 0x00000022, 0x00000000, 0x00040047, 0x0000000a, 0x00000021,
    0x00000000, 0x00030047, 0x0000000b, 0x00000002, 0x00050048, 0x0000000b, 0x00000000, 0x00000023,
    0x00000000, 0x00040047, 0x0000001a, 0x0000000b, 0x0000002a, 0x00040047, 0x0000001d, 0x0000000b,
    0x00000000, 0x00040047, 0x0000001f, 0x0000001e, 0x00000000, 0x00040047, 0x00000033, 0x0000000b,
    0x0000000f, 0x00040047, 0x00000036, 0x0000001e, 0x00000000, 0x00040047, 0x00000039, 0x0000001e,
    0x00000000, 0x00040047, 0x00000041, 0x0000001e, 0x00000000, 0x00040047, 0x00000044, 0x0000000b,
    0x00000000, 0x00040047, 0x00000045, 0x0000001e, 0x00000000, 0x00040047, 0x00000046, 0x0000001e,
    0x00000001, 0x00040047, 0x00000055, 0x0000000b, 0x0000000f, 0x00040047, 0x00000057, 0x0000001e,
    0x00000000, 0x00040047, 0x00000059, 0x0000001e, 0x00000001, 0x00040047, 0x0000005c, 0x0000001e,
    0x00000000, 0x00020013, 0x00000002, 0x00030016, 0x00000004, 0x00000020, 0x00040017, 0x00000003,
    0x00000004, 0x00000004, 0x00040017, 0x00000005, 0x00000004, 0x00000003, 0x00040017, 0x00000006,
    0x00000004, 0x00000002, 0x0004001e, 0x00000007, 0x00000003, 0x00000006, 0x00040015, 0x00000008,
    0x00000020, 0x00000000, 0x0005001e, 0x00000009, 0x00000003, 0x00000004, 0x00000005, 0x0003001e,
    0x0000000b, 0x00000003, 0x00040020, 0x0000000c, 0x00000002, 0x0000000b, 0x0004003b, 0x0000000c,
    0x0000000a, 0x00000002, 0x00040021, 0x00000010, 0x00000003, 0x00000005, 0x00040020, 0x00000011,
    0x00000002, 0x00000003, 0x0004002b, 0x00000008, 0x00000012, 0x00000000, 0x0004002b, 0x00000004,
    0x00000014, 0x3f800000, 0x00040020, 0x0000001b, 0x00000001, 0x00000008, 0x0004003b, 0x0000001b,
    0x0000001a, 0x00000001, 0x00040020, 0x0000001e, 0x00000003, 0x00000003, 0x0004003b, 0x0000001e,
    0x0000001d, 0x00000003, 0x00040020, 0x00000020, 0x00000003, 0x00000006, 0x0004003b, 0x00000020,
    0x0000001f, 0x00000003, 0x00030021, 0x00000022, 0x00000002, 0x0004002b, 0x00000004, 0x00000023,
    0x00000000, 0x0004002b, 0x00000004, 0x00000024, 0x3f000000, 0x0005002c, 0x00000006, 0x00000025,
    0x00000024, 0x00000024, 0x00040020, 0x0000002b, 0x00000003, 0x00000004, 0x0004002b, 0x00000008,
    0x0000002c, 0x00000001, 0x00040020, 0x00000034, 0x00000001, 0x00000003, 0x0004003b, 0x00000034,
    0x00000033, 0x00000001, 0x00040020, 0x00000037, 0x00000001, 0x00000006, 0x0004003b, 0x00000037,
    0x00000036, 0x00000001, 0x0004003b, 0x0000001e, 0x00000039, 0x00000003, 0x00040020, 0x00000042,
    0x00000001, 0x00000004, 0x0004003b, 0x00000042, 0x00000041, 0x00000001, 0x0004003b, 0x0000001e,
    0x00000044, 0x00000003, 0x0004003b, 0x0000002b, 0x00000045, 0x00000003, 0x00040020, 0x00000047,
    0x00000003, 0x00000005, 0x0004003b, 0x00000047, 0x00000046, 0x00000003, 0x0004003b, 0x00000034,
    0x00000055, 0x00000001, 0x0004003b, 0x00000042, 0x00000057, 0x00000001, 0x00040020, 0x0000005a,
    0x00000001, 0x00000005, 0x0004003b, 0x0000005a, 0x00000059, 0x00000001, 0x0004003b, 0x0000001e,
    0x0000005c, 0x00000003, 0x00050036, 0x00000003, 0x0000000f, 0x00000000, 0x00000010, 0x00030037,
    0x00000005, 0x0000000e, 0x000200f8, 0x0000000d, 0x00050041, 0x00000011, 0x00000013, 0x0000000a,
    0x00000012, 0x000200f9, 0x00000015, 0x000200f8, 0x00000015, 0x00050050, 0x00000003, 0x00000016,
    0x0000000e, 0x00000014, 0x0004003d, 0x00000003, 0x00000017, 0x00000013, 0x00050085, 0x00000003,
    0x00000018, 0x00000016, 0x00000017, 0x000200fe, 0x00000018, 0x00010038, 0x00050036, 0x00000002,
    0x00000021, 0x00000000, 0x00000022, 0x000200f8, 0x00000019, 0x0004003d, 0x00000008, 0x0000001c,
    0x0000001a, 0x000200f9, 0x00000026, 0x000200f8, 0x00000026, 0x00040070, 0x00000004, 0x00000027,
    0x0000001c, 0x00070050, 0x00000003, 0x00000028, 0x00000027, 0x00000023, 0x00000023, 0x00000014,
    0x00050050, 0x00000007, 0x00000029, 0x00000028, 0x00000025, 0x00050051, 0x00000003, 0x0000002a,
    0x00000029, 0x00000000, 0x0003003e, 0x0000001d, 0x0000002a, 0x00050041, 0x0000002b, 0x0000002d,
    0x0000001d, 0x0000002c, 0x0004003d, 0x00000004, 0x0000002e, 0x0000002d, 0x0004007f, 0x00000004,
    0x0000002f, 0x0000002e, 0x0003003e, 0x0000002d, 0x0000002f, 0x00050051, 0x00000006, 0x00000030,
    0x00000029, 0x00000001, 0x0003003e, 0x0000001f, 0x00000030, 0x000100fd, 0x00010038, 0x00050036,
    0x00000002, 0x0000003a, 0x00000000, 0x00000022, 0x000200f8, 0x00000031, 0x0004003d, 0x00000003,
    0x00000035, 0x00000033, 0x0004003d, 0x00000006, 0x00000038, 0x00000036, 0x00050050, 0x00000007,
    0x00000032, 0x00000035, 0x00000038, 0x00050041, 0x00000011, 0x0000003b, 0x0000000a, 0x00000012,
    0x000200f9, 0x0000003c, 0x000200f8, 0x0000003c, 0x00050051, 0x00000006, 0x0000003d, 0x00000032,
    0x00000001, 0x00050050, 0x00000005, 0x0000003e, 0x0000003d, 0x00000023, 0x00050039, 0x00000003,
    0x0000003f, 0x0000000f, 0x0000003e, 0x0003003e, 0x00000039, 0x0000003f, 0x000100fd, 0x00010038,
    0x00050036, 0x00000002, 0x00000048, 0x00000000, 0x00000022, 0x000200f8, 0x00000040, 0x0004003d,
    0x00000004, 0x00000043, 0x00000041, 0x000200f9, 0x00000049, 0x000200f8, 0x00000049, 0x00070050,
    0x00000003, 0x0000004a, 0x00000043, 0x00000043, 0x00000043, 0x00000043, 0x00060050, 0x00000005,
    0x0000004b, 0x00000043, 0x00000043, 0x00000043, 0x00060050, 0x00000009, 0x0000004c, 0x0000004a,
    0x00000043, 0x0000004b, 0x00050051, 0x00000003, 0x0000004d, 0x0000004c, 0x00000000, 0x0003003e,
    0x00000044, 0x0000004d, 0x00050041, 0x0000002b, 0x0000004e, 0x00000044, 0x0000002c, 0x0004003d,
    0x00000004, 0x0000004f, 0x0000004e, 0x0004007f, 0x00000004, 0x00000050, 0x0000004f, 0x0003003e,
    0x0000004e, 0x00000050, 0x00050051, 0x00000004, 0x00000051, 0x0000004c, 0x00000001, 0x0003003e,
    0x00000045, 0x00000051, 0x00050051, 0x00000005, 0x00000052, 0x0000004c, 0x00000002, 0x0003003e,
    0x00000046, 0x00000052, 0x000100fd, 0x00010038, 0x00050036, 0x00000002, 0x0000005d, 0x00000000,
    0x00000022, 0x000200f8, 0x00000053, 0x0004003d, 0x00000003, 0x00000056, 0x00000055, 0x0004003d,
    0x00000004, 0x00000058, 0x00000057, 0x0004003d, 0x00000005, 0x0000005b, 0x00000059, 0x00060050,
    0x00000009, 0x00000054, 0x00000056, 0x00000058, 0x0000005b, 0x00050041, 0x00000011, 0x0000005e,
    0x0000000a, 0x00000012, 0x000200f9, 0x0000005f, 0x000200f8, 0x0000005f, 0x00050051, 0x00000005,
    0x00000060, 0x00000054, 0x00000002, 0x00050051, 0x00000004, 0x00000061, 0x00000054, 0x00000001,
    0x0005008e, 0x00000005, 0x00000062, 0x00000060, 0x00000061, 0x00050039, 0x00000003, 0x00000063,
    0x0000000f, 0x00000062, 0x0003003e, 0x0000005c, 0x00000063, 0x000100fd, 0x00010038,
];
const kOpUndef: u32 = 1;
const kOpName: u32 = 5;
const kOpExtInstImport: u32 = 11;
const kOpEntryPoint: u32 = 15;
const kOpExecutionMode: u32 = 16;
const kOpCapability: u32 = 17;
const kOpTypeVoid: u32 = 19;
const kOpTypePipe: u32 = 38;
const kOpConstantTrue: u32 = 41;
const kOpSpecConstantOp: u32 = 52;
const kOpFunction: u32 = 54;
const kOpFunctionCall: u32 = 57;
const kOpVariable: u32 = 59;
const kOpDecorate: u32 = 71;
const kOpMemberDecorate: u32 = 72;
const kOpDecorationGroup: u32 = 73;
const kOpDecorateId: u32 = 332;
const kOpLabel: u32 = 248;
const kDecorationLocation: u32 = 30;
const kDecorationAlignmentId: u32 = 46;
const kCapabilityFunctionPointersINTEL: u32 = 5603;
const kStorageInput: u32 = 1;
const kStorageOutput: u32 = 3;
#[derive(Default)]
struct Shape {
    entries: Vec<String>,
    functions: i32,
    locations: i32,
    executionModes: i32,
    interfaceVariables: i32,
    names: i32,
    targetsDeclared: bool,
}
fn nameOf(words: &[u32]) -> String {
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_ne_bytes()).collect();
    String::from_utf8(bytes[..bytes.iter().position(|b| *b == 0).unwrap()].to_vec()).unwrap()
}
fn shapeOf(words: &[u32]) -> Shape {
    let mut shape = Shape {
        targetsDeclared: true,
        ..Default::default()
    };
    let mut declared = vec![];
    let mut targets = vec![];
    let mut at = 5;
    while at < words.len() {
        let w = &words[at..];
        let length = (w[0] >> 16) as usize;
        let op = w[0] & 0xffff;
        if (kOpTypeVoid..=kOpTypePipe).contains(&op) || op == kOpExtInstImport {
            declared.push(w[1]);
        }
        if (kOpConstantTrue..=kOpSpecConstantOp).contains(&op)
            || matches!(op, kOpUndef | kOpVariable | kOpFunction)
        {
            declared.push(w[2]);
        }
        if matches!(
            op,
            kOpName | kOpDecorate | kOpMemberDecorate | kOpExecutionMode
        ) {
            targets.push(w[1]);
        }
        if op == kOpFunctionCall {
            targets.push(w[3]);
        }
        if op == kOpEntryPoint {
            let name = nameOf(&w[3..length]);
            targets.push(w[2]);
            targets.extend_from_slice(&w[3 + name.len() / 4 + 1..length]);
            shape.entries.push(name);
        }
        shape.functions += i32::from(op == kOpFunction);
        shape.names += i32::from(op == kOpName);
        shape.locations += i32::from(op == kOpDecorate && w[2] == kDecorationLocation);
        shape.executionModes += i32::from(op == kOpExecutionMode);
        shape.interfaceVariables +=
            i32::from(op == kOpVariable && matches!(w[3], kStorageInput | kStorageOutput));
        at += length;
    }
    for target in targets {
        if !declared.contains(&target) {
            shape.targetsDeclared = false;
        }
    }
    shape
}
#[derive(Default)]
struct Landmarks {
    afterImports: usize,
    firstAnnotation: usize,
    firstBody: usize,
    voidType: u32,
    label: u32,
    fogVs: u32,
    dustFs: u32,
}
fn landmarksOf(words: &[u32]) -> Landmarks {
    let mut m = Landmarks::default();
    let mut at = 5;
    while at < words.len() {
        let w = &words[at..];
        let op = w[0] & 0xffff;
        let length = (w[0] >> 16) as usize;
        if op == kOpExtInstImport {
            m.afterImports = at + length;
        }
        if matches!(op, kOpDecorate | kOpMemberDecorate) && m.firstAnnotation == 0 {
            m.firstAnnotation = at;
        }
        if op == kOpTypeVoid {
            m.voidType = w[1];
        }
        if op == kOpLabel && m.label == 0 {
            m.label = w[1];
            m.firstBody = at + length;
        }
        if op == kOpEntryPoint {
            let name = nameOf(&w[3..length]);
            if name == "fog_vs" {
                m.fogVs = w[2];
            }
            if name == "dust_fs" {
                m.dustFs = w[2];
            }
        }
        at += length;
    }
    m
}
fn inserted(at: usize, instruction: &[u32]) -> Vec<u32> {
    let mut words = kTwoPairs.to_vec();
    words[3] += 1;
    words.splice(at..at, instruction.iter().copied());
    words
}
fn cut(words: &[u32], model: u32, name: &str) -> Vec<u32> {
    let mut s = SpirvEntryScratch::default();
    if spirvEntryModule(words, model, Some(name), &mut s) {
        s.words
    } else {
        vec![]
    }
}
fn bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_ne_bytes()).collect()
}
#[test]
fn spirv_entry_split_keeps_what_each_entry_reaches() {
    assert_eq!(spirvEntryPointCount(&bytes(kTwoPairs), kTwoPairs.len()), 4);
    for (model, name, functions, locations, modes, interfaces) in [
        (kSpirvVertexModel, "fog_vs", 1, 1, 0, 3),
        (kSpirvFragmentModel, "fog_fs", 2, 2, 1, 3),
        (kSpirvVertexModel, "dust_vs", 1, 3, 0, 4),
        (kSpirvFragmentModel, "dust_fs", 2, 3, 1, 4),
    ] {
        let s = shapeOf(&cut(kTwoPairs, model, name));
        assert_eq!(s.entries, vec![name]);
        assert_eq!(s.functions, functions);
        assert_eq!(s.locations, locations);
        assert_eq!(s.executionModes, modes);
        assert_eq!(s.interfaceVariables, interfaces);
        assert!(s.targetsDeclared);
    }
}
#[test]
fn spirv_entry_split_reuses_its_scratch() {
    let mut unaligned = vec![0u8; kTwoPairs.len() * 4 + 1];
    unaligned[1..].copy_from_slice(&bytes(kTwoPairs));
    assert_eq!(spirvEntryPointCount(&unaligned[1..], kTwoPairs.len()), 4);
    let mut scratch = SpirvEntryScratch::default();
    let smaller = cut(kTwoPairs, kSpirvFragmentModel, "fog_fs");
    for (words, model, name) in [
        (kTwoPairs, kSpirvVertexModel, "fog_vs"),
        (kTwoPairs, kSpirvFragmentModel, "dust_fs"),
        (smaller.as_slice(), kSpirvFragmentModel, "fog_fs"),
        (kTwoPairs, kSpirvFragmentModel, "fog_fs"),
        (kTwoPairs, kSpirvVertexModel, "dust_vs"),
    ] {
        let cuts = spirvEntryModule(words, model, Some(name), &mut scratch);
        let fresh = cut(words, model, name);
        assert_eq!(cuts, !fresh.is_empty(), "{name}");
        if cuts {
            assert_eq!(scratch.words, fresh, "{name}");
        }
    }
}
#[test]
fn spirv_entry_split_leaves_what_it_cannot_cut() {
    assert!(cut(kTwoPairs, kSpirvFragmentModel, "fog_vs").is_empty());
    assert!(cut(kTwoPairs, kSpirvVertexModel, "missing").is_empty());
    let alone = cut(kTwoPairs, kSpirvVertexModel, "fog_vs");
    assert_eq!(spirvEntryPointCount(&bytes(&alone), alone.len()), 1);
    assert!(cut(&alone, kSpirvVertexModel, "fog_vs").is_empty());
    let m = landmarksOf(kTwoPairs);
    assert!(
        m.afterImports != 0
            && m.firstAnnotation != 0
            && m.firstBody != 0
            && m.voidType != 0
            && m.label != 0
            && m.fogVs != 0
            && m.dustFs != 0
    );
    let fresh = kTwoPairs[3];
    let relaxedPrecision = 0;
    assert!(
        cut(
            &inserted(
                m.firstAnnotation,
                &[(3 << 16) | kOpDecorate, m.label, relaxedPrecision]
            ),
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
    assert!(
        cut(
            &inserted(
                m.firstAnnotation,
                &[
                    (4 << 16) | kOpDecorateId,
                    m.fogVs,
                    kDecorationAlignmentId,
                    m.dustFs
                ]
            ),
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
    assert!(
        cut(
            &inserted(m.firstAnnotation, &[(2 << 16) | kOpDecorationGroup, fresh]),
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
    let s = shapeOf(&cut(
        &inserted(
            m.firstAnnotation,
            &[(3 << 16) | kOpName, m.label, u32::from(b'x')],
        ),
        kSpirvVertexModel,
        "fog_vs",
    ));
    assert_eq!(s.entries, vec!["fog_vs"]);
    assert_eq!(s.names, 0);
    assert!(s.targetsDeclared);
    let mut set = [0u8; 24];
    set[..20].copy_from_slice(b"OpenCL.DebugInfo.100");
    let mut import = vec![(8 << 16) | kOpExtInstImport, fresh];
    import.extend(
        set.chunks_exact(4)
            .map(|w| u32::from_ne_bytes(w.try_into().unwrap())),
    );
    assert!(
        cut(
            &inserted(m.afterImports, &import),
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
    assert!(
        cut(
            &inserted(
                5,
                &[(2 << 16) | kOpCapability, kCapabilityFunctionPointersINTEL]
            ),
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
    assert!(
        cut(
            &inserted(
                m.firstAnnotation,
                &[
                    (5 << 16) | kOpEntryPoint,
                    kSpirvVertexModel,
                    m.fogVs,
                    u32::from_ne_bytes(*b"abcd"),
                    u32::from_ne_bytes(*b"efgh")
                ]
            ),
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
    assert!(
        cut(
            &inserted(
                m.firstBody,
                &[(5 << 16) | kOpFunction, m.voidType, fresh, 0, m.voidType]
            ),
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
    let mut overBound = kTwoPairs.to_vec();
    overBound[3] = overBound.len() as u32 + 1;
    assert!(cut(&overBound, kSpirvVertexModel, "fog_vs").is_empty());
    assert!(cut(&[], kSpirvVertexModel, "fog_vs").is_empty());
    assert!(
        cut(
            &kTwoPairs[..kTwoPairs.len() - 1],
            kSpirvVertexModel,
            "fog_vs"
        )
        .is_empty()
    );
}
// Decode the exact pinned public bake. SPIR-V is the visitor's whole-module
// branch: one callback containing all entries and the shared source bytes.
#[test]
fn spirv_entry_split_cuts_every_baked_test_shader() {
    use nuxie_ore_metal::rstb_entry_container::parseWholeModuleContainer;
    use nuxie_render_api::{NullFactory, PersistentFactory};
    use nuxie_runtime::source::{assets::shader_asset::ShaderAsset, factory::RuntimeFactoryHandle};
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("exact pinned RIVE_RUNTIME_DIR"),
    );
    let header = std::fs::read_to_string(root.join("tests/gm/ore_gm_shaders.rstb.hpp")).unwrap();
    fn array<'a>(header: &'a str, name: &str) -> &'a str {
        header
            .split_once(name)
            .unwrap()
            .1
            .split_once('{')
            .unwrap()
            .1
            .split_once('}')
            .unwrap()
            .0
    }
    let count: usize = header
        .split_once("kShaderCount =")
        .unwrap()
        .1
        .split_once(';')
        .unwrap()
        .0
        .trim()
        .parse()
        .unwrap();
    let names: Vec<&str> = array(&header, "kShaderNames[")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_matches('"'))
        .collect();
    let offsets: Vec<usize> = array(&header, "kShaderOffsets[")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();
    let data: Vec<u8> = array(&header, "kShaderData[")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
        .collect();
    let mut multiEntryModules = 0;
    for id in 0..count {
        let name = names[id];
        let mut envelope = vec![0];
        envelope.extend_from_slice(&data[offsets[id]..offsets[id + 1]]);
        let mut factory = PersistentFactory::new(NullFactory);
        let mut asset = ShaderAsset::default();
        assert!(
            asset.decode(
                &envelope,
                &RuntimeFactoryHandle::from_factory(&mut factory).unwrap()
            ),
            "{name}"
        );
        let blob = asset.find_shader(5);
        let mut entries = vec![];
        let mut code = None;
        let mut size = 0;
        let visited = !blob.is_empty()
            && parseWholeModuleContainer(
                Some(blob),
                blob.len() as u32,
                &mut entries,
                Some(&mut code),
                Some(&mut size),
            );
        if visited {
            let words: Vec<u32> = code.unwrap()[..size as usize]
                .chunks_exact(4)
                .map(|w| u32::from_ne_bytes(w.try_into().unwrap()))
                .collect();
            if spirvEntryPointCount(&bytes(&words), words.len()) >= 2 {
                multiEntryModules += 1;
                for entry in entries {
                    if entry.stage > 1 {
                        continue;
                    }
                    let physical = std::str::from_utf8(entry.physical.as_bytes()).unwrap();
                    let shape = shapeOf(&cut(
                        &words,
                        if entry.stage == 0 {
                            kSpirvVertexModel
                        } else {
                            kSpirvFragmentModel
                        },
                        physical,
                    ));
                    assert_eq!(shape.entries, vec![physical], "{name}: {physical}");
                    assert!(shape.targetsDeclared, "{name}: {physical}");
                }
            }
        }
        assert!(visited, "{name}");
    }
    assert!(multiEntryModules > 0);
}
