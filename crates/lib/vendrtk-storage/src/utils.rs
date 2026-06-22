use crate::error::Result;
use sha2::{Digest, Sha256};

pub fn hash_md5(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

pub fn hash_bytes(bytes: &[u8]) -> Result<String> {
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}