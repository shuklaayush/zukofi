use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use tfhe::zk::CompactPkeCrs;

pub fn write_crs_to_file(crs: &CompactPkeCrs, filepath: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::create(filepath)?;
    let mut writer = BufWriter::new(file);
    bincode::serialize_into(&mut writer, crs)?;
    writer.flush()?;
    Ok(())
}

pub fn read_crs_from_file(filepath: &Path) -> Result<CompactPkeCrs, Box<dyn Error>> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let crs = bincode::deserialize_from(reader)?;
    Ok(crs)
}
