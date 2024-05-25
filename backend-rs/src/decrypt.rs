use tfhe::{prelude::FheDecrypt, FheUint64};

use crate::setup::ServerSetupConfig;

pub trait Decrypter {
    fn decrypt(&self, cipher: FheUint64) -> u64;
}

impl Decrypter for ServerSetupConfig {
    fn decrypt(&self, cipher: FheUint64) -> u64 {
        cipher.decrypt(&self.client_key)
    }
}
