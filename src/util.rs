use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::OsRng;

pub fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as _
}

pub fn fill_bytes(key: &mut [u8]) {
    OsRng.fill_bytes(key);
}
