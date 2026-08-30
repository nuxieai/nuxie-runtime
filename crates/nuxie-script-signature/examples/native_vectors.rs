//! Produce independent vectors with the original, native libhydrogen signer.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    libhydrogen::init().unwrap();
    let keypair =
        libhydrogen::sign::KeyPair::gen_deterministic(&libhydrogen::sign::Seed::from([0x42; 32]));
    let public_key: [u8; 32] = keypair.public_key.into();
    let mut vectors = Vec::new();
    for context in [*b"RiveCode", *b"Context2"] {
        for length in [0, 1, 15, 16, 17, 31, 32, 33, 64, 257, 4097] {
            let message: Vec<u8> = (0..length).map(|index| (index % 251) as u8).collect();
            let signature: [u8; 64] = libhydrogen::sign::create(
                &message,
                &libhydrogen::sign::Context::from(context),
                &keypair.secret_key,
            )
            .unwrap()
            .into();
            assert!(nuxie_script_signature::verify(
                &signature,
                &message,
                &context,
                &public_key
            ));
            vectors.push(serde_json::json!({
                "signature": signature.as_slice(), "message": message,
                "context": context, "publicKey": public_key,
            }));
        }
    }
    println!("{}", serde_json::to_string(&vectors).unwrap());
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("native vector production must use the independent native signer");
}
