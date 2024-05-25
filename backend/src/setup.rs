use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{error::Error, fs::File};
use tfhe::{
    shortint::parameters::PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS_TUNIFORM_2M40,
    zk::{CompactPkeCrs, CompactPkePublicParams},
    ClientKey, CompactPublicKey, CompressedCompactPublicKey, ConfigBuilder, Seed, ServerKey,
};

use super::crs::{read_crs_from_file, write_crs_to_file};

const SEED: u128 = 0;

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerSetupConfig {
    pub client_key: ClientKey,
    pub server_key: ServerKey,
    pub public_key: CompactPublicKey,
    pub public_zk_params: Option<CompactPkePublicParams>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientSetupConfig {
    pub public_key: CompactPublicKey,
    pub public_zk_params: Option<CompactPkePublicParams>,
}

pub fn setup(
    max_num_message: usize,
) -> Result<(ServerSetupConfig, ClientSetupConfig), Box<dyn Error>> {
    let params = PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS_TUNIFORM_2M40;

    // 1. Generate keys
    let client_key = tracing::info_span!("generate client key").in_scope(|| {
        ClientKey::generate_with_seed(
            ConfigBuilder::with_custom_parameters(params, None),
            Seed(SEED),
        )
    });
    let server_key =
        tracing::info_span!("generate server key").in_scope(|| ServerKey::new(&client_key));

    let pubkey_compressed = tracing::info_span!("generate compressed public key")
        .in_scope(|| CompressedCompactPublicKey::new(&client_key));
    let public_key =
        tracing::info_span!("decompress public key").in_scope(|| pubkey_compressed.decompress());

    // 2. Generate or load crs
    // let crs_path = format!("../config/crs/params_{}.bin", max_num_message);
    // let crs_path = Path::new(crs_path.as_str());
    // let crs = if crs_path.exists() {
    //     tracing::info_span!("load crs").in_scope(|| read_crs_from_file(crs_path))?
    // } else {
    //     // This is done in an offline phase and the CRS is shared to all clients and the server
    //     let crs = tracing::info_span!("generate crs")
    //         .in_scope(|| CompactPkeCrs::from_shortint_params(params, max_num_message).unwrap());
    //     write_crs_to_file(&crs, crs_path)?;
    //     crs
    // };

    let client_config = ClientSetupConfig {
        public_key: public_key.clone(),
        // public_zk_params: Some(crs.public_params().clone()),
        public_zk_params: None,
    };

    let server_config = ServerSetupConfig {
        client_key,
        server_key,
        public_key,
        // public_zk_params: Some(crs.public_params().clone()),
        public_zk_params: None,
    };

    Ok((server_config, client_config))
}
