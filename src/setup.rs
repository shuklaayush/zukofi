use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;
use tfhe::zk::CompactPkeCrs;

use super::crs::{read_crs_from_file, write_crs_to_file};

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerSetupConfig {
    pub client_key: tfhe::ClientKey,
    pub server_key: tfhe::ServerKey,
    pub public_key: tfhe::CompactPublicKey,
    pub public_zk_params: tfhe::zk::CompactPkePublicParams,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientSetupConfig {
    pub public_key: tfhe::CompactPublicKey,
    pub public_zk_params: tfhe::zk::CompactPkePublicParams,
}

pub fn setup(
    max_num_message: usize,
) -> Result<(ServerSetupConfig, ClientSetupConfig), Box<dyn Error>> {
    let params =
        tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS_TUNIFORM_2M40;

    // 1. Generate keys
    let client_key = tracing::info_span!("generate client key").in_scope(|| {
        tfhe::ClientKey::generate(tfhe::ConfigBuilder::with_custom_parameters(params, None))
    });
    let server_key =
        tracing::info_span!("generate server key").in_scope(|| tfhe::ServerKey::new(&client_key));
    let public_key = tracing::info_span!("generate public key")
        .in_scope(|| tfhe::CompactPublicKey::try_new(&client_key).unwrap());

    // 2. Generate crs
    let crs_path = format!("crs/crs_{}.bin", max_num_message);
    let crs_path = Path::new(crs_path.as_str());
    let crs = if crs_path.exists() {
        tracing::info_span!("load crs").in_scope(|| read_crs_from_file(crs_path))?
    } else {
        // This is done in an offline phase and the CRS is shared to all clients and the server
        let crs = tracing::info_span!("generate crs")
            .in_scope(|| CompactPkeCrs::from_shortint_params(params, max_num_message).unwrap());
        write_crs_to_file(&crs, crs_path)?;
        crs
    };

    let client_config = ClientSetupConfig {
        public_key: public_key.clone(),
        public_zk_params: crs.public_params().clone(),
    };

    let server_config = ServerSetupConfig {
        client_key,
        server_key,
        public_key,
        public_zk_params: crs.public_params().clone(),
    };

    Ok((server_config, client_config))
}
