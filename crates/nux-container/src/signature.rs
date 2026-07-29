use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Verifier as _, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::NuxPackage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignatureEnvelopeV1 {
    pub version: u32,
    pub signs: String,
    pub algorithm: String,
    pub key_id: String,
    pub signature_base64: String,
}

pub trait ManifestSigner {
    fn sign_manifest(&self, manifest_bytes: &[u8]) -> SignatureEnvelopeV1;
}

impl<F> ManifestSigner for F
where
    F: Fn(&[u8]) -> SignatureEnvelopeV1,
{
    fn sign_manifest(&self, manifest_bytes: &[u8]) -> SignatureEnvelopeV1 {
        self(manifest_bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureVerification {
    Verified { key_id: String },
    UnknownKey,
    BadSignature,
    MalformedEnvelope,
}

pub fn verify_signature<I, K>(package: &NuxPackage<'_>, keys: I) -> SignatureVerification
where
    I: IntoIterator<Item = (K, [u8; 32])>,
    K: AsRef<str>,
{
    let Ok(envelope) = serde_json::from_slice::<SignatureEnvelopeV1>(package.signature_bytes())
    else {
        return SignatureVerification::MalformedEnvelope;
    };
    if envelope.version != 1
        || envelope.signs != "manifest"
        || envelope.algorithm != "ed25519"
        || envelope.key_id.is_empty()
    {
        return SignatureVerification::MalformedEnvelope;
    }

    let Ok(signature_bytes) = STANDARD.decode(&envelope.signature_base64) else {
        return SignatureVerification::MalformedEnvelope;
    };
    let Ok(signature) = Signature::from_slice(&signature_bytes) else {
        return SignatureVerification::MalformedEnvelope;
    };
    let Some((_, public_key)) = keys
        .into_iter()
        .find(|(key_id, _)| key_id.as_ref() == envelope.key_id)
    else {
        return SignatureVerification::UnknownKey;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&public_key) else {
        return SignatureVerification::BadSignature;
    };

    if verifying_key
        .verify(package.manifest_bytes(), &signature)
        .is_ok()
    {
        SignatureVerification::Verified {
            key_id: envelope.key_id,
        }
    } else {
        SignatureVerification::BadSignature
    }
}
