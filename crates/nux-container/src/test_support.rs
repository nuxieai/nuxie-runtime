//! Deterministic key material used only by the fixture corpus and tests.
//!
//! This key is public test data. It must never authorize production packages.

use std::sync::LazyLock;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer as _, SigningKey};

use crate::{ManifestSigner, SignatureEnvelopeV1};

pub const TEST_ONLY_DEV_KEY_ID: &str = "TEST_ONLY_DEV_KEYPAIR";
pub const TEST_ONLY_DEV_KEY_SEED: [u8; 32] = [0x42; 32];

pub static TEST_ONLY_DEV_KEYPAIR: LazyLock<TestOnlyDevKeypair> =
    LazyLock::new(TestOnlyDevKeypair::new);

pub struct TestOnlyDevKeypair {
    signing_key: SigningKey,
}

impl TestOnlyDevKeypair {
    fn new() -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&TEST_ONLY_DEV_KEY_SEED),
        }
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

impl ManifestSigner for TestOnlyDevKeypair {
    fn sign_manifest(&self, manifest_bytes: &[u8]) -> SignatureEnvelopeV1 {
        let signature = self.signing_key.sign(manifest_bytes);
        SignatureEnvelopeV1 {
            version: 1,
            signs: "manifest".to_owned(),
            algorithm: "ed25519".to_owned(),
            key_id: TEST_ONLY_DEV_KEY_ID.to_owned(),
            signature_base64: STANDARD.encode(signature.to_bytes()),
        }
    }
}
