use super::*;

fn assert_backends(
    signature: &[u8; SIGNATURE_BYTES],
    message: &[u8],
    context: &[u8; CONTEXT_BYTES],
    public_key: &[u8; PUBLIC_KEY_BYTES],
    expected: bool,
) {
    assert_eq!(verify(signature, message, context, public_key), expected);
    assert_eq!(
        freestanding_verify(signature, message, context, public_key),
        expected
    );
}

#[test]
fn exact_c_verifier_matches_original_native_library() {
    libhydrogen::init().unwrap();
    let keypair =
        libhydrogen::sign::KeyPair::gen_deterministic(&libhydrogen::sign::Seed::from([0x42; 32]));
    let public_key: [u8; PUBLIC_KEY_BYTES] = keypair.public_key.into();
    for context in [*b"RiveCode", *b"Context2"] {
        for length in [0, 1, 15, 16, 17, 31, 32, 33, 64, 257, 4097] {
            let message: Vec<u8> = (0..length).map(|index| (index % 251) as u8).collect();
            let signature: [u8; SIGNATURE_BYTES] = libhydrogen::sign::create(
                &message,
                &libhydrogen::sign::Context::from(context),
                &keypair.secret_key,
            )
            .unwrap()
            .into();
            assert_backends(&signature, &message, &context, &public_key, true);
            for index in 0..SIGNATURE_BYTES {
                let mut changed = signature;
                changed[index] ^= 1;
                assert_backends(&changed, &message, &context, &public_key, false);
            }
            let mut changed_message = message.clone();
            changed_message.push(1);
            assert_backends(&signature, &changed_message, &context, &public_key, false);
            let mut changed_context = context;
            changed_context[0] ^= 1;
            assert_backends(&signature, &message, &changed_context, &public_key, false);
            let mut changed_key = public_key;
            changed_key[0] ^= 1;
            assert_backends(&signature, &message, &context, &changed_key, false);
            assert_backends(
                &[0; SIGNATURE_BYTES],
                &message,
                &context,
                &public_key,
                false,
            );
        }
    }
}
