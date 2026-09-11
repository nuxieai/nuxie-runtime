import {createCompiler, LANGUAGE_VERSION, type CompileResult, type DesignDocument} from '@nuxie/html-to-riv';

const document: DesignDocument = {
  languageVersion:LANGUAGE_VERSION, html:'<div></div>', css:'', width:390, height:320,
  assets:{photo:{kind:'image',bytes:new Uint8Array([1,2,3])}},
};
async function consume(bytes:Uint8Array) {
  const compiler = await createCompiler(bytes);
  const result:CompileResult = compiler.compile(document);
  if (result.ok) {
    const riv:Uint8Array = result.riv;
    const textId:number | null = result.sourceMap[0].text_run_id;
    const breaks:Array<{id:string;path:string;text_offset:number}> = result.sourceMap[0].text_breaks;
    const textIds:number[] = result.sourceMap[0].text_run_ids;
    const transform: {source:string;rendered:string;scalar_offsets:number[]} | undefined = result.sourceMap[0].text_transform;
    void transform;
    const version:1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 = result.runtimeRequirements.version;
    const capabilities: Array<"layout-css-linear-gradient-v1" | "layout-css-group-opacity-v1" | "layout-css-corner-radii-v1" | "layout-css-border-sides-v1" | "layout-css-solid-borders-v1" | "layout-css-overflow-clip-margin-v1" | "layout-css-axis-overflow-v1" | "layout-css-stacking-v1" | "layout-css-absolute-position-v1" | "layout-css-positioned-paint-v1" | "layout-css-percentage-spacing-v1" | "layout-css-aspect-ratio-pair-v1" | "layout-css-aspect-ratio-v1" | "layout-css-content-box-v1" | "layout-css-indefinite-basis-v1" | "layout-css-intrinsic-sizing-v1" | "layout-css-partial-flex-factors-v1" | "layout-css-flex-factors-v1" | "layout-css-distributed-spacing-v1" | "layout-css-align-content-v1" | "layout-css-align-self-v1" | "layout-css-paint-order-v1" | "layout-css-pixel-bounds-v1" | "text-css-single-line-ellipsis-v1" | "text-solid-strikethroughs-v1" | "text-css-shaping-precision-v1" | "text-solid-underlines-v1" | "text-css-wrapped-tabs-v1" | "text-css-pre-wrap-v1" | "text-css-pre-line-v1" | "text-css-normal-wrap-v1" | "text-css-tabs-v1" | "text-css-nowrap-alignment-v1" | "text-cluster-spacing-v1" | "text-css-letter-spacing-v1" | "text-preserved-space-breaks-v1"> = result.runtimeRequirements.capabilities;
    if (result.runtimeRequirements.version === 26) {
      const gradients: import('@nuxie/html-to-riv').LayoutLinearGradientRequirement[] = result.runtimeRequirements.layout_linear_gradients;
      const direction: import('@nuxie/html-to-riv').LinearGradientDirection = gradients[0].direction;
      void direction;
    }
    if (result.runtimeRequirements.version === 25) {
      const groups: import('@nuxie/html-to-riv').LayoutGroupOpacityRequirement[] = result.runtimeRequirements.layout_group_opacity;
      const alpha: number = groups[0].opacity;
      void alpha;
    }
    if (result.runtimeRequirements.version === 24) {
      const radii: import('@nuxie/html-to-riv').LayoutCornerRadiiRequirement[] = result.runtimeRequirements.layout_corner_radii;
      const axis: import('@nuxie/html-to-riv').CornerRadiusValue = radii[0].radii[0][0];
      void axis;
    }
    if (result.runtimeRequirements.version === 23) {
      const sides: {object_id:number;colors:[number,number,number,number]}[] = result.runtimeRequirements.layout_border_sides;
      void sides;
    }
    if (result.runtimeRequirements.version === 22) {
      const borders: {object_id:number;color:number}[] = result.runtimeRequirements.layout_borders;
      const margins: {object_id:number;origin:"content-box"|"padding-box"|"border-box";pixels:number}[] | undefined = result.runtimeRequirements.layout_overflow_clip_margins;
      void borders; void margins;
    }
    if (result.runtimeRequirements.version === 21) {
      const margins: {object_id:number;origin:"content-box"|"padding-box"|"border-box";pixels:number}[] = result.runtimeRequirements.layout_overflow_clip_margins;
      void margins;
    }
    if (result.runtimeRequirements.version === 20) {
      const clips: {object_id:number;axis:"x"|"y"}[] = result.runtimeRequirements.layout_axis_overflow;
      void clips;
    }
    if (result.runtimeRequirements.version === 19) {
      const contexts: {object_id:number;level:number}[] = result.runtimeRequirements.layout_stacking;
      void contexts;
    }
    if (result.runtimeRequirements.version === 9 || result.runtimeRequirements.version === 10) {
      const factors: import("../js/index.mjs").LayoutFlexFactorsRequirement[] = result.runtimeRequirements.layout_flex_factors;
      void factors;
    }
    if (result.runtimeRequirements.version === 8) {
      const distributed: import("../js/index.mjs").LayoutJustifyContentRequirement[] | undefined = result.runtimeRequirements.layout_justify_content;
      void distributed;
    }
    if (result.runtimeRequirements.version === 7) {
      const lines: Array<{object_id:number;alignment:import('../js/index.mjs').AlignContent}> = result.runtimeRequirements.layout_align_content;
      void lines;
    }
    if (result.runtimeRequirements.version === 6) {
      const aligned: Array<{object_id:number;alignment:import('../js/index.mjs').AlignSelf}> = result.runtimeRequirements.layout_align_self;
      void aligned;
    }
    if (result.runtimeRequirements.version === 5) {
      const layouts:number[] = result.runtimeRequirements.layout_pixel_bounds;
      void layouts;
    }
    if (result.runtimeRequirements.version === 2) {
      const policies:Array<{object_id:number;policy:"css-single-line-ellipsis-v1" | "css-nowrap-alignment-v1" | "css-pre-wrap-v1" | "css-pre-line-v1" | "css-normal-wrap-v1"}> = result.runtimeRequirements.text_policies;
      void policies;
    }
    if (result.runtimeRequirements.version === 4) {
      const strikes:Array<{object_id:number;lines:Array<{color:number;thickness:number;offset:number;line_baseline:number}>}> = result.runtimeRequirements.text_strikethroughs;
      void strikes;
    }
    if (result.runtimeRequirements.version === 3) {
      const lines:Array<{object_id:number;lines:Array<{color:number;thickness:number;offset:number;skip_ink:"none" | "auto" | "all"}>}> = result.runtimeRequirements.text_underlines;
      void lines;
    }
    return {riv,textId,textIds,breaks,version,capabilities};
  }
  const code:string = result.diagnostics[0].code;
  // @ts-expect-error failed compilations have no artifact
  result.riv;
  return code;
}
// @ts-expect-error unknown language versions cannot be authored accidentally
const wrongVersion:DesignDocument = {...document,languageVersion:'future'};
void consume;
void wrongVersion;

declare const spacingRequirements: Extract<import("../js/index.mjs").RuntimeRequirements, {version:16}>;
const spacingTargets: number[] = spacingRequirements.layout_percentage_spacing;
void spacingTargets;

declare const positionedRequirements: Extract<import("../js/index.mjs").RuntimeRequirements, {version:17}>;
const positionedTargets: number[] = positionedRequirements.layout_positioned;
const positionedSpacing: number[] | undefined = positionedRequirements.layout_percentage_spacing;
void positionedTargets; void positionedSpacing;

// Axis transport cannot encode two units at once or omit both units.
// @ts-expect-error ambiguous radius unit
const ambiguousRadius: import('@nuxie/html-to-riv').CornerRadiusValue = {pixels:1,percent:2};
// @ts-expect-error missing radius unit
const missingRadius: import('@nuxie/html-to-riv').CornerRadiusValue = {};
void ambiguousRadius; void missingRadius;

// @ts-expect-error one direction representation is required
const ambiguousGradientDirection: import('@nuxie/html-to-riv').LinearGradientDirection = {degrees:90,corner:{right:true,bottom:true}};
// @ts-expect-error stops retain one unit, not both
const ambiguousGradientPosition: import('@nuxie/html-to-riv').LinearGradientPosition = {pixels:10,percent:50};
