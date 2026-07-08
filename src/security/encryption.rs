pub trait EncryptionAlgorithm {
    fn id(&self) -> u8;
    fn encrypt(&self, key: &[u8], data: &[u8], iv: &[u8]) -> std::io::Result<Vec<u8>>;
    fn decrypt(&self, key: &[u8], data: &[u8], iv: &[u8]) -> std::io::Result<Vec<u8>>;
}

pub struct Aes256Gcm;
impl EncryptionAlgorithm for Aes256Gcm {
    fn id(&self) -> u8 { 1 }
    fn encrypt(&self, _key: &[u8], data: &[u8], _iv: &[u8]) -> std::io::Result<Vec<u8>> {
        // Placeholder
        Ok(data.to_vec())
    }
    fn decrypt(&self, _key: &[u8], data: &[u8], _iv: &[u8]) -> std::io::Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}

pub struct ChaCha20Poly1305;
impl EncryptionAlgorithm for ChaCha20Poly1305 {
    fn id(&self) -> u8 { 2 }
    fn encrypt(&self, _key: &[u8], data: &[u8], _iv: &[u8]) -> std::io::Result<Vec<u8>> {
        // Placeholder
        Ok(data.to_vec())
    }
    fn decrypt(&self, _key: &[u8], data: &[u8], _iv: &[u8]) -> std::io::Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}

pub struct EncryptionManager;

impl EncryptionManager {
    pub fn get_algorithm(id: u8) -> Option<Box<dyn EncryptionAlgorithm>> {
        match id {
            1 => Some(Box::new(Aes256Gcm)),
            2 => Some(Box::new(ChaCha20Poly1305)),
            _ => None,
        }
    }
}
