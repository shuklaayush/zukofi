use std::error::Error;
use tfhe::zk::ZkComputeLoad;
use tfhe::ProvenCompactFheUint64;

use crate::setup::ClientSetupConfig;

pub trait Encrypter {
    fn encrypt_and_prove(&self, clear: u64) -> Result<ProvenCompactFheUint64, Box<dyn Error>>;
}

impl Encrypter for ClientSetupConfig {
    fn encrypt_and_prove(&self, clear: u64) -> Result<ProvenCompactFheUint64, Box<dyn Error>> {
        let cipher = tfhe::ProvenCompactFheUint64::try_encrypt(
            clear,
            &self.public_zk_params,
            &self.public_key,
            ZkComputeLoad::Proof,
        )?;

        Ok(cipher)
    }
}
