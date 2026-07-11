//! Rive signed-content envelope, mirroring C++ `nuxie::SignedContentHeader`
//! (`include/rive/signed_content_header.hpp`).
//!
//! Every text-backed asset payload (`ScriptAsset` Luau bytecode, `ShaderAsset`
//! RSTB blobs) is wrapped as:
//!
//! ```text
//! [flags:1] [signature:64 if signed] [content:N]
//! ```
//!
//! Flags byte: bits 0-6 = version (0-127), bit 7 = isSigned.

/// Size of the libhydrogen signature in bytes (`hydro_sign_BYTES`).
pub const SIGNATURE_SIZE: usize = 64;

/// Borrowed view of a parsed signed-content envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedContent<'a> {
    /// Envelope version (flags bits 0-6).
    pub version: u8,
    /// Signature bytes if the isSigned flag (bit 7) is set.
    pub signature: Option<&'a [u8; SIGNATURE_SIZE]>,
    /// The inner content (for `ScriptAsset`: raw Luau bytecode).
    pub content: &'a [u8],
}

/// Errors from [`SignedContent::parse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeError {
    /// Zero-length payload: not even a flags byte.
    Empty,
    /// isSigned was set but fewer than 65 bytes were present.
    TruncatedSignature,
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvelopeError::Empty => write!(f, "signed-content envelope is empty"),
            EnvelopeError::TruncatedSignature => {
                write!(f, "signed-content envelope truncated before signature end")
            }
        }
    }
}

impl std::error::Error for EnvelopeError {}

impl<'a> SignedContent<'a> {
    /// Parses the envelope. Matches C++ `SignedContentHeader::isValid()`:
    /// the data must contain the flags byte plus, when signed, the 64-byte
    /// signature. Zero-length content is valid.
    pub fn parse(data: &'a [u8]) -> Result<Self, EnvelopeError> {
        let (&flags, rest) = data.split_first().ok_or(EnvelopeError::Empty)?;
        let signed = flags & 0x80 != 0;
        let version = flags & 0x7f;

        if !signed {
            return Ok(SignedContent {
                version,
                signature: None,
                content: rest,
            });
        }

        if rest.len() < SIGNATURE_SIZE {
            return Err(EnvelopeError::TruncatedSignature);
        }
        let (signature, content) = rest.split_at(SIGNATURE_SIZE);
        Ok(SignedContent {
            version,
            signature: Some(signature.try_into().expect("split_at length")),
            content,
        })
    }

    /// True when a signature is present. Verification (libhydrogen
    /// `hydro_sign_verify` against Rive's public key) is out of scope for
    /// the spike; unsigned in-band bytecode is the corpus norm.
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unsigned_envelope() {
        let data = [0x03, 0xaa, 0xbb];
        let parsed = SignedContent::parse(&data).unwrap();
        assert_eq!(parsed.version, 3);
        assert!(!parsed.is_signed());
        assert_eq!(parsed.content, &[0xaa, 0xbb]);
    }

    #[test]
    fn parses_signed_envelope() {
        let mut data = vec![0x81];
        data.extend(std::iter::repeat_n(0x11, SIGNATURE_SIZE));
        data.extend_from_slice(&[1, 2, 3]);
        let parsed = SignedContent::parse(&data).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.signature.unwrap().len(), SIGNATURE_SIZE);
        assert_eq!(parsed.content, &[1, 2, 3]);
    }

    #[test]
    fn empty_content_is_valid_like_cpp() {
        let parsed = SignedContent::parse(&[0x00]).unwrap();
        assert_eq!(parsed.content, &[] as &[u8]);
    }

    #[test]
    fn rejects_empty_and_truncated() {
        assert_eq!(SignedContent::parse(&[]), Err(EnvelopeError::Empty));
        assert_eq!(
            SignedContent::parse(&[0x80, 0x01]),
            Err(EnvelopeError::TruncatedSignature)
        );
    }
}
