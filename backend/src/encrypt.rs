use std::error::Error;
use tfhe::prelude::FheTryEncrypt;
use tfhe::zk::ZkComputeLoad;
use tfhe::{CompactFheUint64, ProvenCompactFheUint64};

use crate::setup::ClientSetupConfig;

pub trait Encrypter {
    fn encrypt(&self, clear: u64) -> Result<CompactFheUint64, Box<dyn Error>>;

    fn encrypt_and_prove(&self, clear: u64) -> Result<ProvenCompactFheUint64, Box<dyn Error>>;
}

impl Encrypter for ClientSetupConfig {
    fn encrypt_and_prove(&self, clear: u64) -> Result<ProvenCompactFheUint64, Box<dyn Error>> {
        let cipher = ProvenCompactFheUint64::try_encrypt(
            clear,
            &self.public_zk_params,
            &self.public_key,
            ZkComputeLoad::Proof,
        )?;

        Ok(cipher)
    }

    fn encrypt(&self, clear: u64) -> Result<CompactFheUint64, Box<dyn Error>> {
        let cipher = CompactFheUint64::try_encrypt(clear, &self.public_key)?;

        Ok(cipher)
    }
}
