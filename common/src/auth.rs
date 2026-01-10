use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::str;
use subtle::ConstantTimeEq;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption error")]
    DecryptionError,
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Decoding error: {0}")]
    DecodeError(#[from] base64::DecodeError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String, // Base64 encoded
    pub salt: String,          // Base64 encoded
}

pub fn hash_password(password: &str) -> (String, String) {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let salt_b64 = general_purpose::STANDARD.encode(salt);

    let mut dk = [0u8; 32]; // 256-bit key
    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, 600_000, &mut dk)
        .expect("HMAC should not fail");
    let hash_b64 = general_purpose::STANDARD.encode(dk);

    (hash_b64, salt_b64)
}

pub fn verify_password(password: &str, hash: &str, salt: &str) -> bool {
    let salt_bytes = match general_purpose::STANDARD.decode(salt) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let hash_bytes = match general_purpose::STANDARD.decode(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };

    let mut dk = [0u8; 32];
    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt_bytes, 600_000, &mut dk)
        .expect("HMAC should not fail");

    // Constant time comparison
    dk.ct_eq(&hash_bytes).into()
}

/// Encrypts the list of users into a base64 encoded string using AES-256-GCM
pub fn encrypt_users(users: &[User], key: &str) -> Result<String, AuthError> {
    let json = serde_json::to_string(users)?;
    let key_bytes = derive_key(key);
    let cipher = Aes256Gcm::new(&key_bytes);

    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let nonce_obj = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_obj, json.as_bytes())
        .map_err(|_| AuthError::EncryptionError)?;

    // Format: nonce + ciphertext
    let mut payload = nonce.to_vec();
    payload.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(payload))
}

/// Decrypts the list of users from a base64 encoded string using AES-256-GCM
pub fn decrypt_users(encrypted_data: &str, key: &str) -> Result<Vec<User>, AuthError> {
    let payload = general_purpose::STANDARD.decode(encrypted_data)?;

    if payload.len() < 12 {
        return Err(AuthError::DecryptionError);
    }

    let (nonce, ciphertext) = payload.split_at(12);
    let nonce_obj = Nonce::from_slice(nonce);

    let key_bytes = derive_key(key);
    let cipher = Aes256Gcm::new(&key_bytes);

    let plaintext = cipher
        .decrypt(nonce_obj, ciphertext)
        .map_err(|_| AuthError::DecryptionError)?;

    let json = str::from_utf8(&plaintext)?;
    let users = serde_json::from_str(json)?;

    Ok(users)
}

// Derive a 32-byte key from the provided secret string using SHA-256
fn derive_key(key: &str) -> aes_gcm::Key<Aes256Gcm> {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    *aes_gcm::Key::<Aes256Gcm>::from_slice(&result)
}
