use crc32fast::Hasher as Crc32Hasher;
use xxhash_rust::xxh64::Xxh64;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChecksumAlgorithm {
    None = 0,
    Crc32c = 1,
    XxHash64 = 2,
    Sha256 = 3,
}

impl ChecksumAlgorithm {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => ChecksumAlgorithm::Crc32c,
            2 => ChecksumAlgorithm::XxHash64,
            3 => ChecksumAlgorithm::Sha256,
            _ => ChecksumAlgorithm::None,
        }
    }
}

pub fn calculate_checksum(algo: ChecksumAlgorithm, data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    match algo {
        ChecksumAlgorithm::None => {}
        ChecksumAlgorithm::Crc32c => {
            let mut hasher = Crc32Hasher::new();
            hasher.update(data);
            let csum = hasher.finalize();
            result[0..4].copy_from_slice(&csum.to_le_bytes());
        }
        ChecksumAlgorithm::XxHash64 => {
            let mut hasher = Xxh64::new(0);
            hasher.update(data);
            let csum = hasher.digest();
            result[0..8].copy_from_slice(&csum.to_le_bytes());
        }
        ChecksumAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            let csum = hasher.finalize();
            result.copy_from_slice(&csum);
        }
    }
    result
}

pub fn verify_checksum(algo: ChecksumAlgorithm, data: &[u8], expected: &[u8; 32]) -> bool {
    if algo == ChecksumAlgorithm::None {
        return true;
    }
    let computed = calculate_checksum(algo, data);
    
    // Only compare the relevant bytes depending on algorithm
    match algo {
        ChecksumAlgorithm::None => true,
        ChecksumAlgorithm::Crc32c => &computed[0..4] == &expected[0..4],
        ChecksumAlgorithm::XxHash64 => &computed[0..8] == &expected[0..8],
        ChecksumAlgorithm::Sha256 => &computed == expected,
    }
}
