use std::error::Error;
use tfhe::{FheUint64, ProvenCompactFheUint64};

use crate::setup::ServerSetupConfig;

pub trait Verifier {
    fn verify_and_expand(
        &self,
        compact: ProvenCompactFheUint64,
    ) -> Result<FheUint64, Box<dyn Error>>;
}

impl Verifier for ServerSetupConfig {
    fn verify_and_expand(
        &self,
        compact: ProvenCompactFheUint64,
    ) -> Result<FheUint64, Box<dyn Error>> {
        let expanded = compact.verify_and_expand(&self.public_zk_params, &self.public_key)?;

        Ok(expanded)
    }
}
