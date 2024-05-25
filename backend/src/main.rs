#[macro_use]
extern crate rocket;

mod crs;
mod decrypt;
mod encrypt;
mod setup;
mod verify;

use rocket::{http::Method, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use setup::ServerSetupConfig;
use tfhe::{set_server_key, CompactPublicKey};
use tracing_forest::{util::LevelFilter, ForestLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::{decrypt::Decrypter, encrypt::Encrypter, setup::setup, verify::Verifier};

#[get("/public-key")]
fn get_public_key(state: &State<ServerSetupConfig>) -> Vec<u8> {
    let mut buffer = Vec::new();
    bincode::serialize_into(&mut buffer, &state.public_key)
        .expect("Failed to serialize public key");
    buffer
}

// #[post("/vote")]
// fn hello(name: &str, age: u8) -> String {
//     // 3. Server side computation
//     let result = {
//         // TODO: Move somewhere else
//         set_server_key(server_config.server_key.clone());

//         // Verify the ciphertexts
//         let a = tracing::info_span!("verify encryption of a")
//             .in_scope(|| server_config.verify_and_expand(a))?;
//         let b = tracing::info_span!("verify encryption of b")
//             .in_scope(|| server_config.verify_and_expand(b))?;

//         tracing::info_span!("calculate result a + b").in_scope(|| a + b)
//     };

//     // 4. Finalize
//     let a_plus_b = tracing::info_span!("decrypt result").in_scope(|| server_config.decrypt(result));

//     // 5. Check the result
//     assert_eq!(a_plus_b, clear_a + clear_b);

//     Ok(())
// }

fn make_cors() -> Cors {
    // let allowed_origins = AllowedOrigins::some_exact(&[
    //     "http://localhost:3000",
    // ]);
    let allowed_origins = AllowedOrigins::all();

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Post].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin",
            "Content-Type",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap()
}

#[launch]
fn rocket() -> _ {
    // 0. Set up tracing
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // 1. Setup
    let (server_config, _) = tracing::info_span!("Setup phase")
        .in_scope(|| setup(1))
        .expect("Failed to setup");

    rocket::build()
        .mount("/", routes![get_public_key])
        .attach(make_cors())
        .manage(server_config)
}
