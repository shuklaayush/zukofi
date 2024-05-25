use tfhe::prelude::FheDecrypt;
use tfhe::set_server_key;
use tfhe::zk::{CompactPkeCrs, ZkComputeLoad};
use tracing_forest::{util::LevelFilter, ForestLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. Set up tracing
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // 1. Setup
    let max_num_message = 1;

    let params =
        tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS_TUNIFORM_2M40;
    let client_key = tracing::info_span!("generate client key").in_scope(|| {
        tfhe::ClientKey::generate(tfhe::ConfigBuilder::with_custom_parameters(params, None))
    });

    // This is done in an offline phase and the CRS is shared to all clients and the server
    let crs = tracing::info_span!("generate crs")
        .in_scope(|| CompactPkeCrs::from_shortint_params(params, max_num_message).unwrap());
    let public_zk_params = crs.public_params();
    let server_key = tfhe::ServerKey::new(&client_key);
    let public_key = tracing::info_span!("generate public key")
        .in_scope(|| tfhe::CompactPublicKey::try_new(&client_key).unwrap());

    // 2. Client side encryption
    let clear_a = 1;
    let clear_b = 2;

    let a = tracing::info_span!("provably encrypt a").in_scope(|| {
        tfhe::ProvenCompactFheUint64::try_encrypt(
            clear_a,
            public_zk_params,
            &public_key,
            ZkComputeLoad::Proof,
        )
    })?;
    let b = tracing::info_span!("provably encrypt b").in_scope(|| {
        tfhe::ProvenCompactFheUint64::try_encrypt(
            clear_b,
            public_zk_params,
            &public_key,
            ZkComputeLoad::Proof,
        )
    })?;

    // 3. Server side computation
    let result = {
        set_server_key(server_key);

        // Verify the ciphertexts
        let a = tracing::info_span!("verify encryption of a")
            .in_scope(|| a.verify_and_expand(public_zk_params, &public_key))?;
        let b = tracing::info_span!("verify encryption of b")
            .in_scope(|| b.verify_and_expand(public_zk_params, &public_key))?;

        tracing::info_span!("calculate result a + b").in_scope(|| a + b)
    };

    // 4. Finalize
    let a_plus_b: u64 =
        tracing::info_span!("decrypt result").in_scope(|| result.decrypt(&client_key));

    // 5. Check the result
    assert_eq!(a_plus_b, clear_a + clear_b);

    Ok(())
}
