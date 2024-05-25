#[macro_use]
extern crate rocket;

mod crs;
mod decrypt;
mod encrypt;
mod setup;
mod verify;

use std::sync::RwLock;

use rocket::{data::ToByteUnit, http::Method, Data, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use setup::ServerSetupConfig;
use tfhe::{prelude::FheTryTrivialEncrypt, set_server_key, CompactFheUint64, FheUint64};
use tracing_forest::{util::LevelFilter, ForestLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::{decrypt::Decrypter, setup::setup, verify::Verifier};

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

pub struct ServerState {
    pub config: ServerSetupConfig,
    pub count: RwLock<FheUint64>,
}

#[get("/public-key")]
fn get_public_key(state: &State<ServerState>) -> Vec<u8> {
    let mut buffer = Vec::new();
    bincode::serialize_into(&mut buffer, &state.config.public_key)
        .expect("Failed to serialize public key");
    buffer
}

#[post("/vote", data = "<data>")]
async fn post_vote<'a>(state: &State<ServerState>, data: Data<'a>) {
    // TODO: Verify inclusion proof

    // Read the data into a byte buffer
    let buffer = data
        .open(10.megabytes())
        .into_bytes()
        .await
        .expect("Failed to read vote data");
    if buffer.is_complete() {
        let buffer = buffer.into_inner();
        // Deserialize the ciphertext
        let cipher: CompactFheUint64 =
            bincode::deserialize(&buffer).expect("Failed to deserialize vote");

        // Expand the ciphertext
        let vote = state.config.expand(cipher);

        // Set the server key
        set_server_key(state.config.server_key.clone());
        // Add the vote to the tally
        println!("Adding vote to tally");
        let mut count = state.count.write().unwrap();
        *count += vote;
    } else {
        println!("there are bytes remaining in the stream");
    }
}

#[get("/finalize")]
fn finalize(state: &State<ServerState>) {
    // TODO: Add time condition or something

    let count = state.count.read().unwrap();
    let count = state.config.decrypt(count.clone());
    println!("Final count: {}", count);
}

#[launch]
fn rocket() -> _ {
    // 0. Set up tracing
    // let env_filter = EnvFilter::builder()
    //     .with_default_directive(LevelFilter::INFO.into())
    //     .from_env_lossy();
    // Registry::default()
    //     .with(env_filter)
    //     .with(ForestLayer::default())
    //     .init();

    // 1. Setup
    let (server_config, _) = tracing::info_span!("Setup phase")
        .in_scope(|| setup(1))
        .expect("Failed to setup");

    // Set the server key
    set_server_key(server_config.server_key.clone());
    let zero = FheUint64::try_encrypt_trivial(0u64).unwrap();

    let state = ServerState {
        config: server_config,
        count: RwLock::new(zero),
    };

    rocket::build()
        .mount("/", routes![get_public_key, post_vote, finalize])
        .attach(make_cors())
        .manage(state)
}
