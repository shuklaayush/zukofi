use tfhe::prelude::FheDecrypt;
use tfhe::set_server_key;
use tfhe::zk::{CompactPkeCrs, ZkComputeLoad};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup
    let max_num_message = 1;

    let params =
        tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS_TUNIFORM_2M40;
    let client_key =
        tfhe::ClientKey::generate(tfhe::ConfigBuilder::with_custom_parameters(params, None));

    // This is done in an offline phase and the CRS is shared to all clients and the server
    let crs = CompactPkeCrs::from_shortint_params(params, max_num_message).unwrap();
    let public_zk_params = crs.public_params();
    let server_key = tfhe::ServerKey::new(&client_key);
    let public_key = tfhe::CompactPublicKey::try_new(&client_key).unwrap();

    // 2. Client side encryption
    let clear_a = 1;
    let clear_b = 2;

    let a = tfhe::ProvenCompactFheUint64::try_encrypt(
        clear_a,
        public_zk_params,
        &public_key,
        ZkComputeLoad::Proof,
    )?;
    let b = tfhe::ProvenCompactFheUint64::try_encrypt(
        clear_b,
        public_zk_params,
        &public_key,
        ZkComputeLoad::Proof,
    )?;

    // 3. Server side computation
    let result = {
        set_server_key(server_key);

        // Verify the ciphertexts
        let a = a.verify_and_expand(public_zk_params, &public_key)?;
        let b = b.verify_and_expand(public_zk_params, &public_key)?;

        a + b
    };

    // 4. Finalize
    let a_plus_b: u64 = result.decrypt(&client_key);

    // 5. Check the result
    assert_eq!(a_plus_b, clear_a + clear_b);

    Ok(())
}
