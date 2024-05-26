use tfhe::{prelude::FheDecrypt, FheBool, FheUint64};

use crate::setup::ServerSetupConfig;

pub trait Decrypter {
    fn decrypt(&self, cipher: FheUint64) -> u64;
    fn decrypt_bool(&self, cipher: FheBool) -> bool;
}

impl Decrypter for ServerSetupConfig {
    fn decrypt(&self, cipher: FheUint64) -> u64 {
        cipher.decrypt(&self.client_key)
    }

    fn decrypt_bool(&self, cipher: FheBool) -> bool {
        cipher.decrypt(&self.client_key)
    }
}
