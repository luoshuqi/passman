use std::collections::HashMap;
use std::sync::Arc;

use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::aes::cipher::InvalidLength;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use argon2::Argon2;
use conerror::Error;

#[derive(Clone)]
pub struct EncryptionManager {
    default_encryptor: u32,
    encryptors: Arc<HashMap<u32, Box<dyn Encryptor + Send + Sync>>>,
}

impl EncryptionManager {
    pub fn new(encryptors: Vec<Box<dyn Encryptor + Send + Sync>>) -> Self {
        assert!(!encryptors.is_empty());

        let id = encryptors[0].id();
        let mut map = HashMap::new();
        for v in encryptors {
            map.insert(v.id(), v);
        }

        EncryptionManager {
            default_encryptor: id,
            encryptors: Arc::new(map),
        }
    }

    fn derive_key(
        &self,
        password: &[u8],
        salt: &[u8],
        key_size: usize,
    ) -> conerror::Result<Vec<u8>> {
        let mut key = vec![0; key_size];
        Argon2::default()
            .hash_password_into(password, salt, &mut key)
            .map_err(Error::plain)?;
        Ok(key)
    }

    pub fn encrypt(&self, data: &[u8], password: &[u8], salt: &[u8]) -> conerror::Result<Vec<u8>> {
        let encryptor = &self.encryptors[&self.default_encryptor];
        let key = self.derive_key(password, salt, encryptor.key_size())?;
        let mut data = encryptor.encrypt(data, &key)?;
        data.extend_from_slice(&encryptor.id().to_le_bytes());
        Ok(data)
    }

    pub fn decrypt(&self, data: &[u8], password: &[u8], salt: &[u8]) -> conerror::Result<Vec<u8>> {
        match self.find_encryptor(data) {
            Some(v) => v.decrypt(
                &data[..data.len() - ENCRYPTOR_ID_SIZE],
                &self.derive_key(password, salt, v.key_size())?,
            ),
            None => Err(Error::plain("unknown encryptor")),
        }
    }

    fn find_encryptor(&self, data: &[u8]) -> Option<&(dyn Encryptor + Send + Sync)> {
        if data.len() < ENCRYPTOR_ID_SIZE {
            return None;
        }
        let mut bytes = [0u8; ENCRYPTOR_ID_SIZE];
        bytes.copy_from_slice(&data[data.len() - ENCRYPTOR_ID_SIZE..]);
        let id = u32::from_le_bytes(bytes);
        self.encryptors.get(&id).map(|v| &**v)
    }
}

const ENCRYPTOR_ID_SIZE: usize = 4;

pub trait Encryptor {
    fn id(&self) -> u32;

    fn key_size(&self) -> usize;

    fn encrypt(&self, data: &[u8], key: &[u8]) -> conerror::Result<Vec<u8>>;

    fn decrypt(&self, data: &[u8], key: &[u8]) -> conerror::Result<Vec<u8>>;
}

#[derive(Copy, Clone)]
pub struct Aes256GcmEncryptor;

impl Aes256GcmEncryptor {
    const KEY_SIZE: usize = 32;
    const NONCE_SIZE: usize = 12;
    const ID: u32 = 1;
}

impl Encryptor for Aes256GcmEncryptor {
    fn id(&self) -> u32 {
        Self::ID
    }

    fn key_size(&self) -> usize {
        Self::KEY_SIZE
    }

    fn encrypt(&self, data: &[u8], key: &[u8]) -> conerror::Result<Vec<u8>> {
        if key.len() != Self::KEY_SIZE {
            return Err(Error::plain(InvalidLength));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let mut data = cipher.encrypt(&nonce, data).map_err(Error::plain)?;
        data.extend_from_slice(&nonce);
        Ok(data)
    }

    fn decrypt(&self, data: &[u8], key: &[u8]) -> conerror::Result<Vec<u8>> {
        if key.len() != Self::KEY_SIZE {
            return Err(Error::plain(InvalidLength));
        }
        if data.len() <= Self::NONCE_SIZE {
            return Err(Error::plain(InvalidLength));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&data[data.len() - Self::NONCE_SIZE..]);
        cipher
            .decrypt(nonce, &data[..data.len() - Self::NONCE_SIZE])
            .map_err(Error::plain)
    }
}

#[cfg(test)]
mod tests {
    use crate::encryption::{Aes256GcmEncryptor, EncryptionManager};

    #[test]
    fn test_encrypt() {
        let plaintext = b"foobar";
        let manager = EncryptionManager::new(vec![Box::new(Aes256GcmEncryptor)]);
        let ciphertext = manager
            .encrypt(plaintext, b"12345678", b"87654321")
            .unwrap();
        assert_eq!(
            plaintext.as_slice(),
            manager
                .decrypt(&ciphertext, b"12345678", b"87654321")
                .unwrap()
        );
    }
}
