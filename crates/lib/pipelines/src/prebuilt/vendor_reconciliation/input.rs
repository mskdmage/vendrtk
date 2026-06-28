pub struct VendorReconciliationInput {
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructs_with_filename_and_bytes() {
        let input = VendorReconciliationInput {
            filename: "file.pdf".into(),
            bytes: vec![1, 2],
        };

        assert_eq!(input.filename, "file.pdf");
        assert_eq!(input.bytes, vec![1, 2]);
    }
}
