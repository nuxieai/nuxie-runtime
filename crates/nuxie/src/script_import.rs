//! Cryptographically bound import authority for remote scripted artifacts.

use ed25519_dalek::{Signature, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use crate::ScriptExecutionAuthorization;

/// A sealed decision about whether one exact imported artifact may execute its
/// embedded `ScriptAsset` bytecode.
///
/// Visual-only authority is freely constructible because it cannot execute
/// code. Authenticated authority is minted only after an Ed25519 signature over
/// a manifest that binds the exact artifact size and SHA-256 digest succeeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptImportCapability(ScriptImportAuthority);

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScriptImportAuthority {
    VisualOnly,
    Authenticated {
        artifact_size: u64,
        artifact_sha256: [u8; 32],
    },
}

/// Why authenticated script authority could not be minted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptAuthenticationError {
    InvalidPublicKey,
    InvalidSignature,
    InvalidManifest,
    ArtifactSizeMismatch,
    ArtifactHashMismatch,
}

impl std::fmt::Display for ScriptAuthenticationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPublicKey => "Ed25519 public key is invalid",
            Self::InvalidSignature => "signature does not authenticate the exact manifest bytes",
            Self::InvalidManifest => {
                "signed manifest has no valid RIV size and SHA-256 declaration"
            }
            Self::ArtifactSizeMismatch => "artifact byte length does not match the signed manifest",
            Self::ArtifactHashMismatch => "artifact SHA-256 does not match the signed manifest",
        })
    }
}

impl std::error::Error for ScriptAuthenticationError {}

#[derive(Deserialize)]
struct SignedArtifactManifest {
    riv: SignedRivDeclaration,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignedRivDeclaration {
    sha256: String,
    size_bytes: u64,
}

impl ScriptImportCapability {
    /// Authority for parsing and rendering ordinary visual content while
    /// keeping every embedded script inert.
    pub const fn visual_only() -> Self {
        Self(ScriptImportAuthority::VisualOnly)
    }

    /// Verify a detached Ed25519 signature and bind executable script authority
    /// to the exact artifact declared by the signed manifest.
    pub fn authenticate_ed25519(
        artifact_bytes: &[u8],
        manifest_bytes: &[u8],
        signature_bytes: &[u8],
        public_key_bytes: &[u8; 32],
    ) -> Result<Self, ScriptAuthenticationError> {
        let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
            .map_err(|_| ScriptAuthenticationError::InvalidPublicKey)?;
        let signature = Signature::from_slice(signature_bytes)
            .map_err(|_| ScriptAuthenticationError::InvalidSignature)?;
        verifying_key
            .verify_strict(manifest_bytes, &signature)
            .map_err(|_| ScriptAuthenticationError::InvalidSignature)?;

        let manifest: SignedArtifactManifest = serde_json::from_slice(manifest_bytes)
            .map_err(|_| ScriptAuthenticationError::InvalidManifest)?;
        let artifact_size = u64::try_from(artifact_bytes.len())
            .map_err(|_| ScriptAuthenticationError::ArtifactSizeMismatch)?;
        if artifact_size != manifest.riv.size_bytes {
            return Err(ScriptAuthenticationError::ArtifactSizeMismatch);
        }
        let artifact_sha256: [u8; 32] = Sha256::digest(artifact_bytes).into();
        if !manifest
            .riv
            .sha256
            .eq_ignore_ascii_case(&sha256_hex(&artifact_sha256))
        {
            return Err(ScriptAuthenticationError::ArtifactHashMismatch);
        }

        Ok(Self(ScriptImportAuthority::Authenticated {
            artifact_size,
            artifact_sha256,
        }))
    }

    pub(crate) fn execution_authorization_for(
        &self,
        artifact_bytes: &[u8],
    ) -> Result<ScriptExecutionAuthorization, ScriptAuthenticationError> {
        let ScriptImportAuthority::Authenticated {
            artifact_size,
            artifact_sha256,
        } = &self.0
        else {
            return Ok(ScriptExecutionAuthorization::VisualOnly);
        };
        if u64::try_from(artifact_bytes.len()) != Ok(*artifact_size) {
            return Err(ScriptAuthenticationError::ArtifactSizeMismatch);
        }
        if <[u8; 32]>::from(Sha256::digest(artifact_bytes)) != *artifact_sha256 {
            return Err(ScriptAuthenticationError::ArtifactHashMismatch);
        }
        Ok(ScriptExecutionAuthorization::Authenticated)
    }
}

impl Default for ScriptImportCapability {
    fn default() -> Self {
        Self::visual_only()
    }
}

fn sha256_hex(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer as _, SigningKey};

    fn signed_capability(
        artifact: &[u8],
        declared_artifact: &[u8],
    ) -> Result<ScriptImportCapability, ScriptAuthenticationError> {
        let signing_key = SigningKey::from_bytes(&[11; 32]);
        let manifest = serde_json::to_vec(&serde_json::json!({
            "riv": {
                "sha256": sha256_hex(&Sha256::digest(declared_artifact).into()),
                "sizeBytes": declared_artifact.len(),
            },
        }))
        .expect("manifest encodes");
        let signature = signing_key.sign(&manifest);
        ScriptImportCapability::authenticate_ed25519(
            artifact,
            &manifest,
            &signature.to_bytes(),
            &signing_key.verifying_key().to_bytes(),
        )
    }

    #[test]
    fn visual_only_capability_never_permits_script_execution() {
        assert_eq!(
            ScriptImportCapability::visual_only().execution_authorization_for(b"artifact"),
            Ok(ScriptExecutionAuthorization::VisualOnly)
        );
    }

    #[test]
    fn authenticated_capability_is_bound_to_the_signed_artifact() {
        let capability = signed_capability(b"artifact", b"artifact").expect("authenticate");
        assert_eq!(
            capability.execution_authorization_for(b"artifact"),
            Ok(ScriptExecutionAuthorization::Authenticated)
        );
        assert_eq!(
            capability.execution_authorization_for(b"artifact!"),
            Err(ScriptAuthenticationError::ArtifactSizeMismatch)
        );
    }

    #[test]
    fn signed_manifest_cannot_authorize_different_artifact_bytes() {
        assert_eq!(
            signed_capability(b"artifacX", b"artifact"),
            Err(ScriptAuthenticationError::ArtifactHashMismatch)
        );
    }
}
