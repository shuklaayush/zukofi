#!/usr/bin/env sh

cd backend-rs
cargo run --release &

cd ../backend-node
node index.js &

wait
