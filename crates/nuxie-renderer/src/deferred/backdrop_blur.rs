//! Not in upstream: the blur a box's backdrop takes.
//!
//! A capture holds what drew behind the box, small (`cmd/deferred_session.rs`).
//! This stage records the passes that blur it into the frame's Ore stream,
//! which replays after every canvas and before the Screen. Each axis halves
//! until the blur spans about three of its texels, or the axis is down to four,
//! then a Gaussian runs along x and another along y, the second with CSS's
//! saturate. Every read folds about the box's edges, so the blur sees the box
//! mirrored past them.
//!
//! The one shader is `backdrop_blur.wgsl`. The compiler's built-in shader
//! function compiles it for every target, and its pieces are checked in beside
//! this file, one file each. A build links only the pieces of its own targets.
#![cfg_attr(
    not(any(
        feature = "native-webgpu-experimental",
        feature = "native-webgl2-experimental",
        feature = "native-ore-metal-experimental",
        feature = "native-ore-vulkan-experimental"
    )),
    allow(dead_code)
)]
use crate::deferred::ore::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::rstb_entry_container::{parsePerEntryContainer, parseWholeModuleContainer};
use nuxie_ore_metal::{
    bind_group_layout::{bindingMapForStages, makeBindGroupLayoutFromBindingMap},
    context::{ContextApi, ShaderTarget},
    gpu_resource::AnyResourceHandle,
    rstb_entry_container::RstbEntryView,
    types::*,
};
use nuxie_render_api::RenderCanvasHandle;
use std::cell::RefCell;

/// A blur spans this many of the recipe's texels, a device pixel over a capture
/// scale kept between the two below. An axis halves until the blur spans no
/// more than this many of its own, and the Gaussian left after halving never
/// spans more.
const TEXELS_PER_BLUR: f32 = 3.0;
const SMALLEST_CAPTURE_SCALE: f32 = 1.0 / 16.0;
const LARGEST_CAPTURE_SCALE: f32 = 1.0 / 2.0;
/// An axis this many texels across or fewer halves no further. A blur far past
/// the box then spreads the box's own average over it.
const SHORTEST_AXIS_TO_HALVE: u32 = 4;
/// A blur pass weighs this many texels each side of the one it draws.
const HALF_WIDTH: usize = 9;
/// What a pass reads at most: the Gaussian's texels in pairs, one linear read
/// a pair, five pairs each side.
const TAPS: usize = HALF_WIDTH + 1;
const LABEL: &str = "nuxie.backdrop-blur";

/// The capture's texels on each axis of a box `extent` device pixels across,
/// for a blur of `device_blur` device pixels. Its scale follows the blur
/// smoothly between the two limits, and a blur of 0 takes the largest. Each
/// axis then fits a whole number of texels to the box, so its last texel holds
/// nothing from past the box's edge, which the mirror can't take back out.
pub(crate) fn capture_texels(extent: [f32; 2], device_blur: f32) -> [Texels; 2] {
    let scale =
        (TEXELS_PER_BLUR / device_blur).clamp(SMALLEST_CAPTURE_SCALE, LARGEST_CAPTURE_SCALE);
    extent.map(|side| {
        // A side that ends on a texel's edge, give or take float error, takes
        // none past it.
        let used = (side * scale - 1.0 / 256.0).ceil().max(1.0) as u32;
        Texels {
            texel: side / used as f32,
            used,
        }
    })
}

/// One axis of a canvas a pass reads or draws: the device pixels a texel spans,
/// and how many texels hold this frame's picture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Texels {
    pub texel: f32,
    pub used: u32,
}

/// One pass of the plan, x first in each pair.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlurPass {
    pub source: [Texels; 2],
    pub target: [Texels; 2],
    pub saturate: f32,
    /// Each read: x and y in device pixels from the drawn texel's center, then
    /// its weight.
    pub taps: Vec<[f32; 3]>,
}

/// The passes that blur and saturate a capture of `capture`'s texels. The first
/// reads the capture and each one after reads the one before. A blur of 0 is
/// one plain pass that only saturates. The last pass draws one texel past the
/// picture on each axis, mirrored, so a linear read at the box's edge never
/// reaches what an earlier frame left.
pub(crate) fn blur_passes(capture: [Texels; 2], device_blur: f32, saturate: f32) -> Vec<BlurPass> {
    let mut axes = capture;
    if !(device_blur > 0.0) {
        return vec![BlurPass {
            source: axes,
            target: one_past(axes),
            saturate,
            taps: vec![[0.0, 0.0, 1.0]],
        }];
    }
    // The recipe's texel, a device pixel over the capture scale, and the blur
    // in those texels. Fitting texels to the box moves their edges, not how
    // many halvings the blur takes. The texel is carried, not recovered as
    // the blur over its span, so a blur too large for a float keeps the
    // smallest scale's and its Gaussian stays capped.
    let first_texel = (device_blur / TEXELS_PER_BLUR)
        .clamp(1.0 / LARGEST_CAPTURE_SCALE, 1.0 / SMALLEST_CAPTURE_SCALE);
    let mut recipe_texel = [first_texel; 2];
    let mut spans = [(device_blur * SMALLEST_CAPTURE_SCALE)
        .max(TEXELS_PER_BLUR)
        .min(device_blur * LARGEST_CAPTURE_SCALE); 2];
    let mut passes = Vec::new();
    loop {
        let halve: [bool; 2] = std::array::from_fn(|at| {
            spans[at] > TEXELS_PER_BLUR && axes[at].used > SHORTEST_AXIS_TO_HALVE
        });
        if halve == [false, false] {
            break;
        }
        let target = std::array::from_fn(|at| {
            if halve[at] {
                Texels {
                    texel: 2.0 * axes[at].texel,
                    used: axes[at].used.div_ceil(2),
                }
            } else {
                axes[at]
            }
        });
        passes.push(BlurPass {
            source: axes,
            target,
            saturate: 1.0,
            taps: halving_taps(axes, halve),
        });
        axes = target;
        for at in 0..2 {
            if halve[at] {
                spans[at] /= 2.0;
                recipe_texel[at] *= 2.0;
            }
        }
    }
    let along = |at: usize| {
        let sigma = axis_sigma(
            device_blur,
            capture[at].texel,
            axes[at].texel,
            recipe_texel[at],
        );
        gaussian_taps(at, axes[at].texel, sigma)
    };
    passes.push(BlurPass {
        source: axes,
        target: axes,
        saturate: 1.0,
        taps: along(0),
    });
    passes.push(BlurPass {
        source: axes,
        target: one_past(axes),
        saturate,
        taps: along(1),
    });
    passes
}

fn one_past(axes: [Texels; 2]) -> [Texels; 2] {
    axes.map(|axis| Texels {
        used: axis.used + 1,
        ..axis
    })
}

/// Reads at the centers of the two or four texels a halved texel covers. The
/// last texel of an odd side reads its missing half through the fold.
fn halving_taps(axes: [Texels; 2], halve: [bool; 2]) -> Vec<[f32; 3]> {
    let places = |at: usize| {
        if halve[at] {
            vec![-axes[at].texel / 2.0, axes[at].texel / 2.0]
        } else {
            vec![0.0]
        }
    };
    let (xs, ys) = (places(0), places(1));
    let weight = 1.0 / (xs.len() * ys.len()) as f32;
    ys.iter()
        .flat_map(|&y| xs.iter().map(move |&x| [x, y, weight]))
        .collect()
}

/// The Gaussian still to apply on an axis, in its own texels: what the blur
/// asks for, less what halving from `first` to `texel` has already spread. It
/// spans at most three of the recipe's texels, `recipe_texel` device pixels
/// each, which only an axis that stopped halving short reaches.
fn axis_sigma(device_blur: f32, first: f32, texel: f32, recipe_texel: f32) -> f64 {
    let (blur, first, texel) = (f64::from(device_blur), f64::from(first), f64::from(texel));
    let spread = (texel * texel - first * first) / 12.0;
    let widest = f64::from(TEXELS_PER_BLUR) * f64::from(recipe_texel);
    (blur * blur - spread).max(0.01).sqrt().min(widest) / texel
}

/// A Gaussian of `sigma` texels along axis `at`, as linear reads of texel
/// pairs: (0, 1), (2, 3) and on to (8, 9) on each side, the center's weight
/// halved between its two sides. A pair whose weights both vanish, as the far
/// ones do on a narrow blur, reads nothing.
fn gaussian_taps(at: usize, texel: f32, sigma: f64) -> Vec<[f32; 3]> {
    let spread = sigma.max(0.05);
    let weights: [f64; HALF_WIDTH + 1] =
        std::array::from_fn(|j| (-0.5 * (j as f64 / spread).powi(2)).exp());
    let total = weights[0] + 2.0 * weights[1..].iter().sum::<f64>();
    let mut taps = Vec::with_capacity(TAPS);
    for pair in 0..TAPS / 2 {
        let near = weights[2 * pair] / total * if pair == 0 { 0.5 } else { 1.0 };
        let far = weights[2 * pair + 1] / total;
        let weight = near + far;
        let offset = if weight > 0.0 {
            (2 * pair) as f64 + far / weight
        } else {
            0.0
        };
        for side in [-1.0, 1.0] {
            let along = (side * offset * f64::from(texel)) as f32;
            let [x, y] = if at == 0 { [along, 0.0] } else { [0.0, along] };
            taps.push([x, y, weight as f32]);
        }
    }
    taps
}

/// The values of one pass, laid out as `Pass` in `backdrop_blur.wgsl`.
fn pass_uniforms(pass: &BlurPass, source_size: [u32; 2], extent: [f32; 2]) -> Vec<u8> {
    let mut words = [0u32; 16 + 4 * TAPS];
    let pairs = [
        pass.source.map(|axis| axis.texel),
        pass.source.map(|axis| axis.used as f32),
        source_size.map(|side| side as f32),
        pass.target.map(|axis| axis.texel),
        pass.target.map(|axis| axis.used as f32),
        extent,
    ];
    for (at, pair) in pairs.iter().enumerate() {
        words[2 * at] = pair[0].to_bits();
        words[2 * at + 1] = pair[1].to_bits();
    }
    words[12] = pass.saturate.to_bits();
    words[13] = pass.taps.len().min(TAPS) as u32;
    for (at, tap) in pass.taps.iter().take(TAPS).enumerate() {
        for (word, value) in tap.iter().enumerate() {
            words[16 + 4 * at + word] = value.to_bits();
        }
    }
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

/// The checked-in pieces of the shader for one target.
struct Pieces {
    source: &'static [u8],
    binding_map: &'static [u8],
    /// GLSL's link fixups, for its vertex entry and its fragment entry.
    fixups: Option<[&'static [u8]; 2]>,
}

const TEXTURE_SAMPLER_PAIRS: &[u8] = include_bytes!("backdrop_blur/texture_sampler_pairs.bin");

fn pieces(target: ShaderTarget) -> Option<Pieces> {
    match target {
        #[cfg(feature = "native-webgpu-experimental")]
        ShaderTarget::wgsl => Some(Pieces {
            source: include_bytes!("backdrop_blur/wgsl_source.bin"),
            binding_map: include_bytes!("backdrop_blur/wgsl_binding_map.bin"),
            fixups: None,
        }),
        #[cfg(feature = "native-webgl2-experimental")]
        ShaderTarget::glsl => Some(Pieces {
            source: include_bytes!("backdrop_blur/glsl_source.bin"),
            binding_map: include_bytes!("backdrop_blur/glsl_binding_map.bin"),
            fixups: Some([
                include_bytes!("backdrop_blur/glsl_vertex_fixup.bin"),
                include_bytes!("backdrop_blur/glsl_fragment_fixup.bin"),
            ]),
        }),
        #[cfg(feature = "native-ore-metal-experimental")]
        ShaderTarget::msl => Some(Pieces {
            source: include_bytes!("backdrop_blur/msl_source.bin"),
            binding_map: include_bytes!("backdrop_blur/msl_binding_map.bin"),
            fixups: None,
        }),
        #[cfg(feature = "native-ore-vulkan-experimental")]
        ShaderTarget::spirv => Some(Pieces {
            source: include_bytes!("backdrop_blur/spirv_source.bin"),
            binding_map: include_bytes!("backdrop_blur/spirv_binding_map.bin"),
            fixups: None,
        }),
        _ => None,
    }
}

/// What every pass draws with.
struct BlurShader {
    layout: AnyResourceHandle,
    pipeline: AnyResourceHandle,
    sampler: AnyResourceHandle,
    _modules: [AnyResourceHandle; 2],
}

fn physical_name(view: &RstbEntryView<'_>) -> String {
    String::from_utf8_lossy(view.physical.as_bytes()).into_owned()
}

/// The modules from the checked-in bytes, as `build_shader_entries` makes a
/// script's: one module for the whole shader on WGSL, MSL and SPIR-V, and one
/// an entry on GLSL. These bytes never came from a file, so there is nothing
/// to check them against.
fn make_shader(
    context: &mut dyn ContextApi,
    target: ShaderTarget,
    format: TextureFormat,
) -> Result<BlurShader, String> {
    let pieces = pieces(target).ok_or_else(|| {
        format!(
            "this build has no pieces for shader target {}",
            target as u8
        )
    })?;
    let module =
        |context: &mut dyn ContextApi, code: &[u8], stage: ShaderStage, fixup: Option<&[u8]>| {
            context.clearLastError();
            context
                .makeShaderModule(&ShaderModuleDesc {
                    code: Some(code),
                    codeSize: code.len() as u32,
                    language: match target {
                        ShaderTarget::wgsl => ShaderLanguage::wgsl,
                        _ => ShaderLanguage::glsl,
                    },
                    stage,
                    label: Some(LABEL),
                    bindingMapBytes: Some(pieces.binding_map),
                    bindingMapSize: pieces.binding_map.len() as u32,
                    texSamplerPairBytes: (!TEXTURE_SAMPLER_PAIRS.is_empty())
                        .then_some(TEXTURE_SAMPLER_PAIRS),
                    texSamplerPairSize: TEXTURE_SAMPLER_PAIRS.len() as u32,
                    glFixupBytes: fixup,
                    glFixupSize: fixup.map_or(0, |fixup| fixup.len() as u32),
                    ..ShaderModuleDesc::default()
                })
                .ok_or_else(|| format!("a module: {}", context.lastError()))
        };
    // The vertex entry's module and name, then the fragment entry's.
    let mut entries: [Option<(AnyResourceHandle, String)>; 2] = [None, None];
    let mut views = Vec::new();
    let size = pieces.source.len() as u32;
    if let Some(fixups) = pieces.fixups {
        if !parsePerEntryContainer(Some(pieces.source), size, &mut views) {
            return Err("the GLSL entries don't read".into());
        }
        for view in &views {
            let stage = match view.stage {
                0 => ShaderStage::vertex,
                1 => ShaderStage::fragment,
                _ => continue,
            };
            let code = view.source.ok_or("a GLSL entry has no source")?;
            let made = module(
                &mut *context,
                code,
                stage,
                Some(fixups[view.stage as usize]),
            )?;
            entries[view.stage as usize] = Some((made, physical_name(view)));
        }
    } else {
        let (mut code, mut code_size) = (None, 0);
        if !parseWholeModuleContainer(
            Some(pieces.source),
            size,
            &mut views,
            Some(&mut code),
            Some(&mut code_size),
        ) {
            return Err("the module doesn't read".into());
        }
        let code = code.ok_or("the module has no source")?;
        let made = module(&mut *context, code, ShaderStage::autoDetect, None)?;
        for view in views.iter().filter(|view| view.stage < 2) {
            entries[view.stage as usize] = Some((made.clone(), physical_name(view)));
        }
    }
    let [Some(vertex), Some(fragment)] = entries else {
        return Err("the shader has no vertex or no fragment entry".into());
    };
    let ((vertex, vertex_entry), (fragment, fragment_entry)) = (vertex, fragment);
    let binding_map = bindingMapForStages(vertex.shaderModuleBase(), fragment.shaderModuleBase());
    let layout = makeBindGroupLayoutFromBindingMap(
        context,
        &binding_map,
        0,
        &[],
        vertex.shaderModuleBase(),
        fragment.shaderModuleBase(),
        Some(vertex_entry.as_str()),
        Some(fragment_entry.as_str()),
    )
    .ok_or_else(|| format!("its layout: {}", context.lastError()))?;
    let layouts = [Some(&layout)];
    let mut desc = PipelineDesc {
        vertexModule: Some(&vertex),
        vertexEntryPoint: Some(vertex_entry.as_str()),
        fragmentModule: Some(&fragment),
        fragmentEntryPoint: Some(fragment_entry.as_str()),
        cullMode: CullMode::none,
        colorCount: 1,
        bindGroupLayouts: Some(&layouts),
        bindGroupLayoutCount: 1,
        label: Some(LABEL),
        ..PipelineDesc::default()
    };
    desc.colorTargets[0] = ColorTargetState {
        format,
        blendEnabled: false,
        writeMask: ColorWriteMask::all,
        ..ColorTargetState::default()
    };
    let mut error = String::new();
    let pipeline = context
        .makePipeline(&desc, Some(&mut error))
        .ok_or_else(|| format!("its pipeline: {error}"))?;
    context.clearLastError();
    // Linear within the one level a canvas has. WebGL2 finds a one-level
    // texture incomplete through a sampler that names more.
    let sampler = context
        .makeSampler(&SamplerDesc {
            minFilter: Filter::linear,
            magFilter: Filter::linear,
            mipmapFilter: Filter::nearest,
            wrapU: WrapMode::clampToEdge,
            wrapV: WrapMode::clampToEdge,
            maxLod: 0.0,
            label: Some(LABEL),
            ..SamplerDesc::default()
        })
        .ok_or_else(|| format!("its sampler: {}", context.lastError()))?;
    Ok(BlurShader {
        layout,
        pipeline,
        sampler,
        _modules: [vertex, fragment],
    })
}

enum Shader {
    /// Not asked for yet by a session whose device is known.
    Unmade,
    Made(BlurShader),
    /// This build or this context can't make it, so every box falls back.
    Refused,
}

/// One place in draw order's canvases: the capture, at half the box, and two
/// the passes draw in turn, one texel larger each way for the last pass's
/// extra texel.
#[derive(Clone)]
pub(crate) struct CanvasSet {
    pub capture: RenderCanvasHandle,
    drawn: [RenderCanvasHandle; 2],
    width: u32,
    height: u32,
}

/// The blur stage a session keeps: the shader, made on first use, and the
/// canvases of each live backdrop, kept between frames because strength
/// changes the part in use every frame.
pub(crate) struct BackdropBlur {
    shader: Shader,
    sets: Vec<CanvasSet>,
    used: usize,
    warmed: bool,
    // This frame's views, uniforms and bind groups, and the warm's canvases,
    // held until the frame has replayed.
    frame: Vec<AnyResourceHandle>,
    frame_canvases: Vec<RenderCanvasHandle>,
}

impl Default for BackdropBlur {
    fn default() -> Self {
        Self {
            shader: Shader::Unmade,
            sets: Vec::new(),
            used: 0,
            warmed: false,
            frame: Vec::new(),
            frame_canvases: Vec::new(),
        }
    }
}

impl BackdropBlur {
    /// Whether this session can blur: its capabilities come from a real device
    /// and the shader could be made for its target. The caller checks that a
    /// render context is bound.
    pub(crate) fn ready(&mut self, ore: &RefCell<DeferredOreContext>) -> bool {
        if matches!(self.shader, Shader::Unmade) {
            let caps = *ore.borrow().caps();
            if !caps.featuresKnown {
                return false;
            }
            let made = make_shader(
                &mut *ore.borrow_mut(),
                caps.shaderTarget,
                caps.canvasTargetFormat,
            );
            self.shader = match made {
                Ok(shader) => Shader::Made(shader),
                Err(error) => {
                    eprintln!("rive deferred: the backdrop blur falls back: {error}");
                    Shader::Refused
                }
            };
        }
        matches!(self.shader, Shader::Made(_))
    }

    /// The canvases of the next place in draw order this frame: last frame's,
    /// unless this capture outgrew them, in which case new ones at its size.
    pub(crate) fn next_set(
        &mut self,
        width: u32,
        height: u32,
        make: &mut dyn FnMut(u32, u32) -> Option<RenderCanvasHandle>,
    ) -> Option<CanvasSet> {
        let place = self.used;
        let fits = self
            .sets
            .get(place)
            .is_some_and(|set| set.width >= width && set.height >= height);
        if !fits {
            let set = CanvasSet {
                capture: make(width, height)?,
                drawn: [make(width + 1, height + 1)?, make(width + 1, height + 1)?],
                width,
                height,
            };
            match self.sets.get_mut(place) {
                Some(kept) => *kept = set,
                None => self.sets.push(set),
            }
        }
        self.used += 1;
        Some(self.sets[place].clone())
    }

    /// Records `passes` over `set`'s capture into the Ore stream. Returns the
    /// canvas the last one drew and the device pixels its texel spans on each
    /// axis, or None, having recorded no pass, if the context refused one of
    /// their resources.
    pub(crate) fn record(
        &mut self,
        ore: &RefCell<DeferredOreContext>,
        set: &CanvasSet,
        passes: &[BlurPass],
        extent: [f32; 2],
    ) -> Option<(RenderCanvasHandle, [f32; 2])> {
        self.record_passes(
            ore,
            &set.capture,
            [&set.drawn[0], &set.drawn[1]],
            passes,
            extent,
        )?;
        let last = passes.last()?;
        let drawn = set.drawn[(passes.len() - 1) % 2].clone();
        Some((drawn, last.target.map(|axis| axis.texel)))
    }

    /// Draws the shader once into a 2 by 2 canvas, once a session, so the
    /// driver has it ready before the first capture needs it.
    pub(crate) fn warm(
        &mut self,
        ore: &RefCell<DeferredOreContext>,
        make: &mut dyn FnMut(u32, u32) -> Option<RenderCanvasHandle>,
    ) {
        if self.warmed || !self.ready(ore) {
            return;
        }
        self.warmed = true;
        let (Some(source), Some(target)) = (make(2, 2), make(2, 2)) else {
            return;
        };
        let two = Texels {
            texel: 1.0,
            used: 2,
        };
        let pass = BlurPass {
            source: [two; 2],
            target: [two; 2],
            saturate: 1.0,
            taps: vec![[0.0, 0.0, 1.0]],
        };
        self.record_passes(ore, &source, [&target, &target], &[pass], [2.0, 2.0]);
        self.frame_canvases.extend([source, target]);
    }

    /// The frame has been taken. Spare canvases go, and this frame's resources
    /// with them.
    pub(crate) fn reset_frame(&mut self) {
        self.sets.truncate(self.used);
        self.used = 0;
        self.frame.clear();
        self.frame_canvases.clear();
    }

    fn record_passes(
        &mut self,
        ore: &RefCell<DeferredOreContext>,
        capture: &RenderCanvasHandle,
        drawn: [&RenderCanvasHandle; 2],
        passes: &[BlurPass],
        extent: [f32; 2],
    ) -> Option<()> {
        let Shader::Made(shader) = &self.shader else {
            return None;
        };
        let (layout, pipeline, sampler) = (
            shader.layout.clone(),
            shader.pipeline.clone(),
            shader.sampler.clone(),
        );
        let size = |canvas: &RenderCanvasHandle| {
            let canvas = canvas.borrow();
            [canvas.width(), canvas.height()]
        };
        // Each pass draws the canvas the one before didn't, the first into
        // drawn[0], and reads what the one before drew, the first the capture.
        let drawing = passes.len().min(2);
        let reading = passes.len().saturating_sub(1).min(2);
        let mut context = ore.borrow_mut();
        let capture_view =
            context.recordWrapCanvasImage(nuxie_render_api::canvas_texture_info(capture))?;
        let mut draw_views = Vec::with_capacity(drawing);
        for canvas in &drawn[..drawing] {
            // SAFETY: a recording context with a canvas provider reads only the
            // typed owner `canvas_texture_info` keeps, never its raw pointers.
            let view = unsafe {
                context.wrapCanvasTextureInfo(nuxie_render_api::canvas_texture_info(canvas))
            }?;
            draw_views.push(view);
        }
        let mut read_views = Vec::with_capacity(reading);
        for canvas in &drawn[..reading] {
            read_views.push(
                context.recordWrapCanvasImage(nuxie_render_api::canvas_texture_info(canvas))?,
            );
        }
        let mut groups = Vec::with_capacity(passes.len());
        let mut buffers = Vec::with_capacity(passes.len());
        for (at, pass) in passes.iter().enumerate() {
            let (source, source_size) = match at {
                0 => (&capture_view, size(capture)),
                _ => (&read_views[(at - 1) % 2], size(drawn[(at - 1) % 2])),
            };
            let values = pass_uniforms(pass, source_size, extent);
            context.clearLastError();
            let buffer = context.makeBuffer(&BufferDesc {
                usage: BufferUsage::uniform,
                size: values.len() as u32,
                data: Some(&values),
                immutable: true,
                label: Some(LABEL),
            })?;
            let group = context.makeBindGroup(&BindGroupDesc {
                layout: Some(&layout),
                ubos: &[UBOEntry {
                    slot: 0,
                    buffer: Some(&buffer),
                    offset: 0,
                    size: 0,
                }],
                uboCount: 1,
                textures: &[TexEntry {
                    slot: 1,
                    view: Some(source),
                }],
                textureCount: 1,
                samplers: &[SampEntry {
                    slot: 2,
                    sampler: Some(&sampler),
                }],
                samplerCount: 1,
                label: Some(LABEL),
            });
            let Some(group) = group else {
                eprintln!(
                    "rive deferred: a backdrop blur pass has no bind group: {}",
                    context.lastError()
                );
                return None;
            };
            buffers.push(buffer);
            groups.push(group);
        }
        for (at, pass) in passes.iter().enumerate() {
            let mut desc = RenderPassDesc {
                colorCount: 1,
                label: Some(LABEL),
                ..RenderPassDesc::default()
            };
            desc.colorAttachments[0] = ColorAttachment {
                view: Some(&draw_views[at % 2]),
                resolveTarget: None,
                loadOp: LoadOp::clear,
                storeOp: StoreOp::store,
                clearColor: ClearColor {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            };
            let mut recording = context.beginRenderPass(&desc, None)?;
            recording.setViewport(
                0.0,
                0.0,
                pass.target[0].used as f32,
                pass.target[1].used as f32,
                0.0,
                1.0,
            );
            recording.setPipeline(Some(&pipeline));
            recording.setBindGroup(0, Some(&groups[at]), None, 0);
            recording.draw(3, 1, 0, 0);
            recording.finish();
        }
        self.frame.push(capture_view);
        self.frame.extend(draw_views);
        self.frame.extend(read_views);
        self.frame.extend(buffers);
        self.frame.extend(groups);
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many passes halve axis `at`.
    fn halvings(passes: &[BlurPass], at: usize) -> usize {
        passes
            .iter()
            .filter(|pass| pass.target[at].texel != pass.source[at].texel)
            .count()
    }

    fn close(got: f32, want: f32) -> bool {
        (got - want).abs() <= want.abs() * 1e-5
    }

    /// Halvings and final texels on each axis, x first, against
    /// `blur-stage-sim-5.py`'s per-axis table. That table lists y first, and
    /// its texels are the recipe's: here each one is the capture's fitted
    /// texel times two for each halving.
    #[test]
    fn halvings_follow_the_recipe_on_each_axis() {
        let cases: [([f32; 2], f32, [usize; 2], [u32; 2]); 4] = [
            // The frost at 90 px, captured at 1/16: one halving each way, to
            // about 32 px texels.
            ([1206.0, 2622.0], 90.0, [1, 1], [76, 164]),
            // The frost at 1800 px: x stops at 3 texels after 5, y after 6.
            ([1206.0, 2622.0], 1800.0, [5, 6], [76, 164]),
            // A strip at 600 px: y stops at 3 texels after 3.
            ([1206.0, 270.0], 600.0, [4, 3], [76, 17]),
            // A strip at 900 px.
            ([1206.0, 270.0], 900.0, [5, 3], [76, 17]),
        ];
        for (extent, blur, want, first_used) in cases {
            let capture = capture_texels(extent, blur);
            for at in 0..2 {
                assert_eq!(capture[at].used, first_used[at], "{extent:?} at {blur}");
                assert!(close(capture[at].texel, extent[at] / first_used[at] as f32));
            }
            let passes = blur_passes(capture, blur, 1.8);
            let got = [halvings(&passes, 0), halvings(&passes, 1)];
            assert_eq!(got, want, "{extent:?} at {blur}");
            assert_eq!(passes.len(), want[0].max(want[1]) + 2);
            let last = passes.last().unwrap();
            for at in 0..2 {
                let texel = capture[at].texel * (1u32 << want[at]) as f32;
                assert!(close(last.source[at].texel, texel), "{extent:?} at {blur}");
            }
        }
    }

    /// Up to 48 px the capture alone gets the blur to three texels, so no
    /// axis halves, though fitting texels to the box makes them a little
    /// smaller than the recipe's.
    #[test]
    fn a_blur_the_capture_covers_takes_no_halving() {
        for blur in [6.0, 8.0, 24.0, 30.0, 47.0, 48.0] {
            let capture = capture_texels([1206.0, 2622.0], blur);
            let passes = blur_passes(capture, blur, 1.0);
            assert_eq!(passes.len(), 2, "blur {blur}");
        }
        let capture = capture_texels([1206.0, 2622.0], 48.5);
        assert_eq!(blur_passes(capture, 48.5, 1.0).len(), 3);
    }

    /// A capture's texels tile the box exactly, so none holds what lies past
    /// it, and a box smaller than a texel takes one.
    #[test]
    fn capture_texels_fit_the_box() {
        for (extent, blur) in [
            ([608.0, 300.0], 8.0),
            ([1206.0, 2622.0], 30.0),
            ([300.0, 300.0], 900.0),
            ([1206.0, 32.0], 180.0),
            ([5.0, 3.0], 900.0),
        ] {
            let capture = capture_texels(extent, blur);
            for at in 0..2 {
                assert!(capture[at].used >= 1);
                let covered = capture[at].texel * capture[at].used as f32;
                assert!(close(covered, extent[at]), "{extent:?} at {blur}");
            }
        }
        // 608 px at 8 px is 228 texels of 8/3 px, with nothing to round.
        let capture = capture_texels([608.0, 608.0], 8.0);
        assert_eq!(capture[0].used, 228);
    }

    /// The inputs table's blur row: one too large for a float is only a very
    /// large blur. It plans as the largest float does, each axis halving to
    /// its own stop, and the Gaussian left stays capped at three of the
    /// recipe's texels, 16 device px doubled for each halving. A recipe texel
    /// recovered as the blur over its span is infinity over infinity there,
    /// and the cap falls away to even weights.
    #[test]
    fn a_blur_too_large_for_a_float_is_a_very_large_blur() {
        for extent in [[1206.0, 2622.0], [1206.0, 270.0], [5.0, 3.0]] {
            let capture = capture_texels(extent, f32::INFINITY);
            assert_eq!(capture, capture_texels(extent, f32::MAX), "{extent:?}");
            let passes = blur_passes(capture, f32::INFINITY, 1.8);
            assert_eq!(passes, blur_passes(capture, f32::MAX, 1.8), "{extent:?}");
            for (at, pass) in passes[passes.len() - 2..].iter().enumerate() {
                let texel = pass.source[at].texel;
                assert!(pass.source[at].used <= SHORTEST_AXIS_TO_HALVE);
                let recipe_texel = 16.0 * f64::from(1u32 << halvings(&passes, at));
                let sigma = f64::from(TEXELS_PER_BLUR) * recipe_texel / f64::from(texel);
                assert_eq!(pass.taps, gaussian_taps(at, texel, sigma), "{extent:?}");
            }
        }
    }

    /// Every pass but the last draws as many texels as it reads, or half; the
    /// last draws one past on each axis and alone saturates.
    #[test]
    fn the_last_pass_saturates_and_draws_one_past() {
        let capture = capture_texels([1206.0, 270.0], 600.0);
        let passes = blur_passes(capture, 600.0, 1.8);
        let (last, rest) = passes.split_last().unwrap();
        assert_eq!(last.saturate, 1.8);
        for at in 0..2 {
            assert_eq!(last.target[at].used, last.source[at].used + 1);
            assert_eq!(last.target[at].texel, last.source[at].texel);
        }
        for pass in rest {
            assert_eq!(pass.saturate, 1.0);
        }
        // Each pass reads what the one before drew.
        assert_eq!(passes[0].source, capture);
        for pair in passes.windows(2) {
            assert_eq!(pair[1].source, pair[0].target);
        }
    }

    /// A blur of 0 is one plain pass: one read where the texel is, the
    /// saturate, and one texel past.
    #[test]
    fn no_blur_is_one_plain_pass() {
        let capture = capture_texels([300.0, 200.0], 0.0);
        assert_eq!(capture[0].used, 150);
        for blur in [0.0, -1.0, f32::NAN] {
            let passes = blur_passes(capture, blur, 1.8);
            assert_eq!(passes.len(), 1);
            assert_eq!(passes[0].taps, vec![[0.0, 0.0, 1.0]]);
            assert_eq!(passes[0].saturate, 1.8);
            assert_eq!(passes[0].source, capture);
            assert_eq!(passes[0].target, one_past(capture));
        }
    }

    /// Test 3's wrong build: at 0.4 px the far pairs' weights vanish, and
    /// taking them as 0 / 0 puts a read at a place that isn't a number.
    #[test]
    fn a_tiny_blur_reads_only_real_places() {
        for blur in [0.4, 1.0, 1.11, 8.0, 900.0] {
            let capture = capture_texels([608.0, 300.0], blur);
            for pass in blur_passes(capture, blur, 1.8) {
                assert!(pass.taps.len() <= TAPS);
                let mut total = 0.0;
                for tap in &pass.taps {
                    assert!(
                        tap.iter().all(|value| value.is_finite()),
                        "blur {blur}: {tap:?}"
                    );
                    total += tap[2];
                }
                assert!(
                    (total - 1.0).abs() < 1e-5,
                    "blur {blur}: weights sum to {total}"
                );
            }
        }
    }

    /// Each pair is one linear read, placed so its two texels keep their own
    /// weights: spread back over texels, the reads give the Gaussian.
    #[test]
    fn paired_reads_give_the_gaussian() {
        for sigma in [0.2, 1.0, 2.5, 3.0] {
            let texel = 4.0;
            let taps = gaussian_taps(1, texel, sigma);
            assert_eq!(taps.len(), TAPS);
            let mut kernel = [0.0f64; 2 * HALF_WIDTH + 2];
            for tap in &taps {
                assert_eq!(tap[0], 0.0);
                let place = f64::from(tap[1] / texel) + HALF_WIDTH as f64;
                let low = place.floor();
                let part = place - low;
                kernel[low as usize] += f64::from(tap[2]) * (1.0 - part);
                kernel[low as usize + 1] += f64::from(tap[2]) * part;
            }
            let weights: Vec<f64> = (0..=HALF_WIDTH)
                .map(|j| (-0.5 * (j as f64 / sigma).powi(2)).exp())
                .collect();
            let total = weights[0] + 2.0 * weights[1..].iter().sum::<f64>();
            for (at, got) in kernel.iter().enumerate().take(2 * HALF_WIDTH + 1) {
                let want = weights[at.abs_diff(HALF_WIDTH)] / total;
                assert!(
                    (got - want).abs() < 1e-5,
                    "sigma {sigma}, texel {at}: {got} against {want}"
                );
            }
            assert!(kernel[2 * HALF_WIDTH + 1].abs() < 1e-6);
        }
    }

    /// `Pass` in `backdrop_blur.wgsl`: six pairs, saturate, the tap count,
    /// then the taps from byte 64, 224 bytes in all.
    #[test]
    fn uniforms_match_the_shader_layout() {
        let capture = capture_texels([608.0, 300.0], 8.0);
        let passes = blur_passes(capture, 8.0, 1.8);
        let last = passes.last().unwrap();
        let bytes = pass_uniforms(last, [305, 151], [608.0, 300.0]);
        assert_eq!(bytes.len(), 224);
        let word = |at: usize| u32::from_le_bytes(bytes[4 * at..4 * at + 4].try_into().unwrap());
        let float = |at: usize| f32::from_bits(word(at));
        assert_eq!(float(0), last.source[0].texel);
        assert_eq!(float(3), last.source[1].used as f32);
        assert_eq!([float(4), float(5)], [305.0, 151.0]);
        assert_eq!(float(8), last.target[0].used as f32);
        assert_eq!([float(10), float(11)], [608.0, 300.0]);
        assert_eq!(float(12), 1.8);
        assert_eq!(word(13) as usize, last.taps.len());
        assert_eq!([float(16), float(17), float(18)], last.taps[0]);
    }
}
