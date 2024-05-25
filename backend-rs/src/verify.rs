use std::error::Error;
use tfhe::{CompactFheUint64, FheUint64, ProvenCompactFheUint64};

use crate::setup::ServerSetupConfig;

pub trait Verifier {
    fn expand(&self, compact: CompactFheUint64) -> FheUint64;

    fn verify_and_expand(
        &self,
        compact: ProvenCompactFheUint64,
    ) -> Result<FheUint64, Box<dyn Error>>;
}

impl Verifier for ServerSetupConfig {
    fn expand(&self, compact: CompactFheUint64) -> FheUint64 {
        compact.expand()
    }

    fn verify_and_expand(
        &self,
        compact: ProvenCompactFheUint64,
    ) -> Result<FheUint64, Box<dyn Error>> {
        let public_zk_params = self
            .public_zk_params
            .as_ref()
            .ok_or("zk params not found")?;
        let expanded = compact.verify_and_expand(public_zk_params, &self.public_key)?;

        Ok(expanded)
    }
}
