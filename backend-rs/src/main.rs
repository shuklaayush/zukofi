#[macro_use]
extern crate rocket;

mod crs;
mod decrypt;
mod encrypt;
mod setup;
mod verify;

use std::sync::RwLock;

use rocket::{data::ToByteUnit, http::Method, serde::json::Json, Data, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use serde::{Deserialize, Serialize};
use setup::ServerSetupConfig;
use tfhe::{prelude::FheTryTrivialEncrypt, set_server_key, CompactFheUint64, FheUint64};
use tracing_forest::{util::LevelFilter, ForestLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::{decrypt::Decrypter, setup::setup, verify::Verifier};

const ZUPASS_VERIFY_URL: &str = "http://localhost:8001/verify";

#[derive(Serialize, Deserialize)]
pub struct VoteData {
    pub vote: CompactFheUint64,
    pub pcd: Vec<u8>,
}

pub struct ServerState {
    pub config: ServerSetupConfig,
    pub count: RwLock<FheUint64>,
}

fn make_cors() -> Cors {
    // let allowed_origins = AllowedOrigins::some_exact(&[
    //     "http://localhost:3000",
    // ]);
    let allowed_origins = AllowedOrigins::all();

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
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

#[get("/public-key")]
fn get_public_key(state: &State<ServerState>) -> Vec<u8> {
    let mut buffer = Vec::new();
    bincode::serialize_into(&mut buffer, &state.config.public_key)
        .expect("Failed to serialize public key");
    buffer
}

#[post("/vote", data = "<data>")]
fn post_vote(state: &State<ServerState>, data: Json<VoteData>) {
    // Read the data into a byte buffer
    let data = data.into_inner();

    // Verify the voter
    let client = reqwest::blocking::Client::new();
    let resp = client.post(ZUPASS_VERIFY_URL).body(data.pcd).send();

    if resp.is_ok() {
        // Expand the ciphertext
        let vote = state.config.expand(data.vote);

        // Set the server key
        set_server_key(state.config.server_key.clone());
        // Add the vote to the tally
        println!("Adding vote to tally");
        let mut count = state.count.write().unwrap();
        *count += vote;
    } else {
        println!(
            "Failed to verify inclusion proof: {}",
            resp.unwrap().text().unwrap()
        );
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
