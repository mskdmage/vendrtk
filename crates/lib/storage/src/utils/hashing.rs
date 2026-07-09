use sha2::{Digest, Sha256};

// TODO: hashing function should probably be decoupled from the storage library
pub fn calculate_hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let bytes = b"%PDF-1.4 test";
        let hash = calculate_hash(bytes);
        assert_eq!(
            hash,
            "d663640088750cf16276d623c2588d7233f2b84b45f4b2e20832f47b16aa5618"
        );
    }
}
