use std::error::Error;
use tfhe::set_server_key;
use tracing_forest::{util::LevelFilter, ForestLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

mod crs;
mod decrypt;
mod encrypt;
mod setup;
mod verify;

use crate::{decrypt::Decrypter, encrypt::Encrypter, setup::setup, verify::Verifier};

fn main() -> Result<(), Box<dyn Error>> {
    // 0. Set up tracing
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // 1. Setup
    let (server_config, client_config) =
        tracing::info_span!("Setup phase").in_scope(|| setup(1))?;

    // 2. Client side encryption
    let clear_a = 1;
    let a = tracing::info_span!("provably encrypt a")
        .in_scope(|| client_config.encrypt_and_prove(clear_a))?;

    let clear_b = 2;
    let b = tracing::info_span!("provably encrypt b")
        .in_scope(|| client_config.encrypt_and_prove(clear_b))?;

    // 3. Server side computation
    let result = {
        // TODO: Move somewhere else
        set_server_key(server_config.server_key.clone());

        // Verify the ciphertexts
        let a = tracing::info_span!("verify encryption of a")
            .in_scope(|| server_config.verify_and_expand(a))?;
        let b = tracing::info_span!("verify encryption of b")
            .in_scope(|| server_config.verify_and_expand(b))?;

        tracing::info_span!("calculate result a + b").in_scope(|| a + b)
    };

    // 4. Finalize
    let a_plus_b = tracing::info_span!("decrypt result").in_scope(|| server_config.decrypt(result));

    // 5. Check the result
    assert_eq!(a_plus_b, clear_a + clear_b);

    Ok(())
}
