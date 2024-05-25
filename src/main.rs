use tfhe::prelude::*;
use tfhe::{generate_keys, set_server_key, CompactPublicKey, ConfigBuilder, FheUint8};

fn main() {
    // 1. Setup
    let config = ConfigBuilder::default()
        .use_custom_parameters(
            tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS,
            None,
        )
        .build();
    let (client_key, server_key) = generate_keys(config);
    let public_key = CompactPublicKey::new(&client_key);
    set_server_key(server_key);

    // 2. Operate
    let clear_a = 27u8;
    let clear_b = 128u8;

    let a = FheUint8::try_encrypt(clear_a, &public_key).unwrap();
    let b = FheUint8::try_encrypt(clear_b, &public_key).unwrap();

    let result = a + b;

    // 3. Finalize
    let decrypted_result: u8 = result.decrypt(&client_key);

    // 4. Verify
    let clear_result = clear_a + clear_b;
    assert_eq!(decrypted_result, clear_result);
}
