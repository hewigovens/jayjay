use sha2::{Digest, Sha256};

pub fn hex_sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_to_hex_sha256() {
        assert_eq!(
            hex_sha256(b"jayjay"),
            "1554fea732db61c9bead5b0df7d0d2085ecaad0955aaa22122c2cbc5e2c36c39"
        );
    }
}
