use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{KNOWN_CAPABILITIES, NUX_MAX_EXTERNAL_ASSET_BYTES, NuxContainerError, Result};

const MAX_LIFECYCLE_DURATION_MS: u32 = 60_000;

fn option_vec_is_none_or_empty<T>(value: &Option<Vec<T>>) -> bool {
    value.as_ref().is_none_or(Vec::is_empty)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NuxPackageManifestV1 {
    pub version: u32,
    pub identity: Identity,
    pub producer: Producer,
    pub scene_format: SceneFormat,
    pub required_capabilities: Vec<String>,
    pub scene: SceneMember,
    pub journey: JourneyMember,
    pub entry: Entry,
    pub screens: Vec<Screen>,
    #[serde(skip_serializing_if = "option_vec_is_none_or_empty")]
    pub transitions: Option<Vec<TransitionV1>>,
    pub text_inputs: Vec<TextInput>,
    pub assets: Assets,
    pub members: Vec<MemberInventoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Identity {
    pub experience_id: String,
    pub build_id: String,
    pub app_id: String,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Producer {
    pub compiler_commit: String,
    pub compiler_version: String,
    pub runtime_revision: String,
    pub luau: LuauProducer,
    pub min_runtime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LuauProducer {
    pub revision: String,
    pub bytecode_versions: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SceneFormat {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SceneMember {
    pub member: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JourneyMember {
    pub member: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub schema_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub screen_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Screen {
    pub screen_id: String,
    pub artboard_id: String,
    pub artboard_name: String,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit: Option<ScreenExitV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenExitV1 {
    pub complete_event_name: String,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionKindV1 {
    Choreographed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionEndpointV1 {
    pub complete_event_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionReverseV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incoming_on_top: Option<bool>,
    pub source: TransitionEndpointV1,
    pub destination: TransitionEndpointV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionV1 {
    pub id: String,
    pub kind: TransitionKindV1,
    pub source_screen_id: String,
    pub destination_screen_id: String,
    pub duration_ms: u32,
    pub incoming_on_top: bool,
    pub source: TransitionEndpointV1,
    pub destination: TransitionEndpointV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<TransitionReverseV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextInput {
    pub input_id: String,
    pub screen_id: String,
    pub artboard_id: String,
    pub view_node_id: String,
    pub rendered_node_id: String,
    pub rive_text_object_key: String,
    pub rive_text_run_object_key: String,
    pub rive_text_name: String,
    pub rive_text_run_name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    pub editable: bool,
    pub geometry: Geometry,
    pub style: TextInputStyle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyboard_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure_text_entry: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_field_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Geometry {
    pub x_path: String,
    pub y_path: String,
    pub width_path: String,
    pub height_path: String,
    pub rotation_path: String,
    #[serde(rename = "scaleXPath")]
    pub scale_xpath: String,
    #[serde(rename = "scaleYPath")]
    pub scale_ypath: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextInputStyle {
    pub font_family: String,
    pub font_weight: String,
    pub font_style: FontStyleValue,
    pub font_size: f64,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub color: u32,
    pub font_asset_rive_unique_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontStyleValue {
    Normal,
    Italic,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Assets {
    pub images: Vec<ImageAsset>,
    pub fonts: Vec<FontAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageAsset {
    pub location: AssetLocation,
    pub rive_asset_id: u64,
    pub rive_unique_name: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub content_type: ImageContentType,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FontAsset {
    pub location: AssetLocation,
    pub rive_asset_id: u64,
    pub rive_unique_name: String,
    pub family: String,
    pub weight: String,
    pub style: FontStyle,
    pub sha256: String,
    pub size_bytes: u64,
    pub content_type: FontContentType,
    pub format: FontFormat,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum AssetLocation {
    External { key: String },
    Embedded { member: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageContentType {
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/webp")]
    Webp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontContentType {
    #[serde(rename = "font/ttf")]
    Ttf,
    #[serde(rename = "font/otf")]
    Otf,
    #[serde(rename = "application/octet-stream")]
    OctetStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontFormat {
    Ttf,
    Otf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    Normal,
    Italic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberInventoryEntry {
    pub name: String,
    pub role: MemberRole,
    pub sha256: String,
    pub size_bytes: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Manifest,
    Scene,
    Journey,
    Asset,
}

impl ImageContentType {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

impl FontContentType {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ttf => "font/ttf",
            Self::Otf => "font/otf",
            Self::OctetStream => "application/octet-stream",
        }
    }
}

impl NuxPackageManifestV1 {
    pub(crate) fn validate_structure(&self) -> Result<()> {
        if self.version != 1 {
            return Err(NuxContainerError::InvalidManifest(format!(
                "manifest version {} is not 1",
                self.version
            )));
        }

        validate_required_strings(self)?;
        validate_screens(self)?;
        validate_lifecycle(self)?;
        validate_text_inputs(self)?;
        validate_members(self)?;
        validate_assets(self)?;

        for capability in &self.required_capabilities {
            if !KNOWN_CAPABILITIES.contains(&capability.as_str()) {
                return Err(NuxContainerError::UnknownCapability(capability.clone()));
            }
        }

        Ok(())
    }
}

fn validate_required_strings(manifest: &NuxPackageManifestV1) -> Result<()> {
    let identity = &manifest.identity;
    non_empty(&identity.experience_id, "identity.experienceId")?;
    non_empty(&identity.build_id, "identity.buildId")?;
    non_empty(&identity.app_id, "identity.appId")?;
    non_empty(&identity.environment, "identity.environment")
}

fn validate_screens(manifest: &NuxPackageManifestV1) -> Result<()> {
    let mut ids = HashSet::new();
    for screen in &manifest.screens {
        if !ids.insert(screen.screen_id.as_str()) {
            return invalid("screen ids must be unique");
        }
    }
    if !ids.contains(manifest.entry.screen_id.as_str()) {
        return invalid("entry.screenId does not name a screen");
    }
    Ok(())
}

fn validate_lifecycle(manifest: &NuxPackageManifestV1) -> Result<()> {
    let screen_ids = manifest
        .screens
        .iter()
        .map(|screen| screen.screen_id.as_str())
        .collect::<HashSet<_>>();
    for screen in &manifest.screens {
        let Some(exit) = &screen.exit else {
            continue;
        };
        non_empty(
            &exit.complete_event_name,
            "screens[].exit.completeEventName",
        )?;
        if !(1..=MAX_LIFECYCLE_DURATION_MS).contains(&exit.duration_ms) {
            return invalid("screens[].exit.durationMs must be between 1 and 60000");
        }
    }

    let mut transition_ids = HashSet::new();
    for transition in manifest.transitions.iter().flatten() {
        non_empty(&transition.id, "transitions[].id")?;
        if !transition_ids.insert(transition.id.as_str()) {
            return invalid("transition ids must be unique");
        }
        if !screen_ids.contains(transition.source_screen_id.as_str()) {
            return invalid("transitions[].sourceScreenId does not name a screen");
        }
        if !screen_ids.contains(transition.destination_screen_id.as_str()) {
            return invalid("transitions[].destinationScreenId does not name a screen");
        }
        if !(1..=MAX_LIFECYCLE_DURATION_MS).contains(&transition.duration_ms) {
            return invalid("transitions[].durationMs must be between 1 and 60000");
        }
        non_empty(
            &transition.source.complete_event_name,
            "transitions[].source.completeEventName",
        )?;
        non_empty(
            &transition.destination.complete_event_name,
            "transitions[].destination.completeEventName",
        )?;
        if let Some(reverse) = &transition.reverse {
            if reverse
                .duration_ms
                .is_some_and(|duration_ms| !(1..=MAX_LIFECYCLE_DURATION_MS).contains(&duration_ms))
            {
                return invalid("transitions[].reverse.durationMs must be between 1 and 60000");
            }
            non_empty(
                &reverse.source.complete_event_name,
                "transitions[].reverse.source.completeEventName",
            )?;
            non_empty(
                &reverse.destination.complete_event_name,
                "transitions[].reverse.destination.completeEventName",
            )?;
        }
    }
    Ok(())
}

fn validate_text_inputs(manifest: &NuxPackageManifestV1) -> Result<()> {
    for input in &manifest.text_inputs {
        for (value, field) in [
            (&input.input_id, "textInputs[].inputId"),
            (&input.screen_id, "textInputs[].screenId"),
            (&input.artboard_id, "textInputs[].artboardId"),
            (&input.view_node_id, "textInputs[].viewNodeId"),
            (&input.rendered_node_id, "textInputs[].renderedNodeId"),
            (
                &input.rive_text_object_key,
                "textInputs[].riveTextObjectKey",
            ),
            (
                &input.rive_text_run_object_key,
                "textInputs[].riveTextRunObjectKey",
            ),
            (&input.rive_text_name, "textInputs[].riveTextName"),
            (&input.rive_text_run_name, "textInputs[].riveTextRunName"),
        ] {
            non_empty(value, field)?;
        }

        let geometry = &input.geometry;
        for (value, field) in [
            (&geometry.x_path, "geometry.xPath"),
            (&geometry.y_path, "geometry.yPath"),
            (&geometry.width_path, "geometry.widthPath"),
            (&geometry.height_path, "geometry.heightPath"),
            (&geometry.rotation_path, "geometry.rotationPath"),
            (&geometry.scale_xpath, "geometry.scaleXPath"),
            (&geometry.scale_ypath, "geometry.scaleYPath"),
        ] {
            non_empty(value, field)?;
        }

        let style = &input.style;
        non_empty(&style.font_family, "style.fontFamily")?;
        non_empty(&style.font_weight, "style.fontWeight")?;
        non_empty(
            &style.font_asset_rive_unique_name,
            "style.fontAssetRiveUniqueName",
        )?;
        if !style.font_size.is_finite()
            || style.font_size <= 0.0
            || !style.line_height.is_finite()
            || style.line_height <= 0.0
            || !style.letter_spacing.is_finite()
        {
            return invalid("text input style numeric values are invalid");
        }
        if input.max_length == Some(0) {
            return invalid("textInputs[].maxLength must be positive");
        }
    }
    Ok(())
}

fn validate_members(manifest: &NuxPackageManifestV1) -> Result<()> {
    let mut names = HashSet::new();
    for member in &manifest.members {
        if !names.insert(member.name.as_str()) {
            return invalid("members[] names must be unique");
        }
        if member.name == "manifest" {
            if member.role != MemberRole::Manifest
                || member.sha256 != "0".repeat(64)
                || member.size_bytes != 0
            {
                return invalid("manifest inventory entry must use its zero digest sentinel");
            }
        } else {
            validate_sha256(&member.sha256, "members[].sha256")?;
        }
    }
    Ok(())
}

fn validate_assets(manifest: &NuxPackageManifestV1) -> Result<()> {
    for image in &manifest.assets.images {
        validate_sha256(&image.sha256, "assets.images[].sha256")?;
        validate_asset_location(&image.location, &image.sha256)?;
        validate_external_asset_size(&image.location, image.size_bytes)?;
    }
    for font in &manifest.assets.fonts {
        validate_sha256(&font.sha256, "assets.fonts[].sha256")?;
        validate_asset_location(&font.location, &font.sha256)?;
        validate_external_asset_size(&font.location, font.size_bytes)?;
    }
    validate_sha256(&manifest.scene.sha256, "scene.sha256")?;
    validate_sha256(&manifest.journey.sha256, "journey.sha256")
}

fn validate_external_asset_size(location: &AssetLocation, size_bytes: u64) -> Result<()> {
    if matches!(location, AssetLocation::External { .. })
        && size_bytes > NUX_MAX_EXTERNAL_ASSET_BYTES
    {
        return invalid(&format!(
            "external asset declares {size_bytes} bytes, exceeding the {NUX_MAX_EXTERNAL_ASSET_BYTES}-byte limit"
        ));
    }
    Ok(())
}

fn validate_asset_location(location: &AssetLocation, expected_hash: &str) -> Result<()> {
    let value = match location {
        AssetLocation::External { key } => key,
        AssetLocation::Embedded { member } => member,
    };
    let Some(rest) = value.strip_prefix("assets/sha256/") else {
        return Err(NuxContainerError::InvalidAsset(value.clone()));
    };
    let Some((hash, extension)) = rest.rsplit_once('.') else {
        return Err(NuxContainerError::InvalidAsset(value.clone()));
    };
    const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "ttf", "otf"];
    if hash != expected_hash || !EXTENSIONS.contains(&extension) {
        return Err(NuxContainerError::InvalidAsset(value.clone()));
    }
    Ok(())
}

pub(crate) fn validate_sha256(value: &str, field: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return invalid(&format!("{field} must be 64 lowercase hex characters"));
    }
    Ok(())
}

fn non_empty(value: &str, field: &str) -> Result<()> {
    if value.is_empty() {
        return invalid(&format!("{field} must not be empty"));
    }
    Ok(())
}

fn invalid<T>(message: &str) -> Result<T> {
    Err(NuxContainerError::InvalidManifest(message.to_owned()))
}
