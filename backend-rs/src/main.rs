#[macro_use]
extern crate rocket;

mod crs;
mod decrypt;
mod encrypt;
mod setup;
mod verify;

use std::sync::RwLock;

use rocket::{
    http::{Method, Status},
    serde::json::Json,
    State,
};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use serde::{Deserialize, Serialize};
use setup::ServerSetupConfig;
use tfhe::{
    prelude::{FheOrd, FheTryTrivialEncrypt},
    set_server_key, CompactFheUint64, FheUint64,
};
use tracing_forest::{util::LevelFilter, ForestLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::{decrypt::Decrypter, setup::setup, verify::Verifier};

const NUM_OPTIONS: usize = 3;
const MAX_VOTE_COST: u64 = 10;
const ZUPASS_VERIFY_URL: &str = "http://localhost:8001/verify";

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteData {
    pub votes: Vec<Vec<u8>>,
    pub pcd: String,
}

pub struct ServerState {
    pub config: ServerSetupConfig,
    pub counts: RwLock<Vec<FheUint64>>,
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
fn post_vote(state: &State<ServerState>, data: Json<VoteData>) -> Status {
    // Read the data into a byte buffer
    let data = data.into_inner();
    println!("Received votes: {:}", data.votes.len());

    // 1. Check if number of votes is correct
    // Deserialize the ciphertext
    println!("Deserializing votes");
    let votes: Vec<FheUint64> = data
        .votes
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
            let cipher = bincode::deserialize(&data)
                .expect(format!("Failed to deserialize vote {}", i).as_str());
            state.config.expand(cipher)
        })
        .collect();
    assert_eq!(votes.len(), NUM_OPTIONS, "Invalid number of votes");

    // Set the server key
    set_server_key(state.config.server_key.clone());

    // 2. Check if vote is valid
    // TODO: Should happen in ZK proof on client side
    println!("Checking if vote is valid");
    let vote_cost: FheUint64 = votes.iter().map(|vote| vote * vote).sum();
    let cost_ok = vote_cost.le(MAX_VOTE_COST);
    let cost_ok = state.config.decrypt_bool(cost_ok);
    assert!(cost_ok, "Invalid vote");

    // 3. Verify if the voter is eligible
    println!("Checking if voter is eligible");
    let client = reqwest::blocking::Client::new();
    let pcd_json = &serde_json::json!({
        "pcd": data.pcd
    });
    let resp = client.post(ZUPASS_VERIFY_URL).json(&pcd_json).send();

    if resp.is_ok() {
        if !resp.unwrap().status().is_success() {
            println!("Failed to verify inclusion proof");

            Status::BadRequest
        } else {
            //4. Add the vote to the tally
            println!("Adding votes to tally");
            let mut counts = state.counts.write().unwrap();
            for (i, vote) in votes.iter().enumerate() {
                counts[i] += vote.clone();
            }

            Status::Ok
        }
    } else {
        println!(
            "Failed to contact Zupass verify server: {}",
            resp.unwrap().text().unwrap()
        );
        Status::BadRequest
    }
}

#[get("/finalize")]
fn finalize(state: &State<ServerState>) {
    // TODO: Add time condition or something

    let counts = state.counts.read().unwrap();
    let counts: Vec<_> = counts
        .iter()
        .map(|count| state.config.decrypt(count.clone()))
        .collect();
    for (i, count) in counts.iter().enumerate() {
        println!("Option {}: {}", i, count);
    }
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
    let zeros = (0..NUM_OPTIONS)
        .map(|_| FheUint64::try_encrypt_trivial(0u64).unwrap())
        .collect();

    let state = ServerState {
        config: server_config,
        counts: RwLock::new(zeros),
    };

    rocket::build()
        .mount("/", routes![get_public_key, post_vote, finalize])
        .attach(make_cors())
        .manage(state)
}
