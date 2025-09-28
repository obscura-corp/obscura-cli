use crate::util::errors::{ObscuraError, ObscuraResult};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, Payload},
    Key, XChaCha20Poly1305, XNonce,
};
use rand::{rngs::OsRng, RngCore};
use zeroize::ZeroizeOnDrop;

#[derive(ZeroizeOnDrop)]
pub struct AeadKey([u8; 32]);

impl AeadKey {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Clone for AeadKey {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

pub struct AeadResult {
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

impl AeadResult {
    pub fn encrypt(plaintext: &[u8], key: &AeadKey, aad: &[u8]) -> ObscuraResult<Self> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let nonce_bytes = nonce.as_slice().try_into().unwrap();

        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| ObscuraError::EncryptionFailed)?;

        Ok(Self {
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    pub fn decrypt(
        ciphertext: &[u8],
        key: &AeadKey,
        nonce: &[u8; 24],
        aad: &[u8],
    ) -> ObscuraResult<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = XNonce::from_slice(nonce);

        cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| ObscuraError::DecryptionFailed)
    }
}

pub fn encrypt_with_key(plaintext: &[u8], key: &AeadKey, aad: &[u8]) -> ObscuraResult<AeadResult> {
    AeadResult::encrypt(plaintext, key, aad)
}

pub fn decrypt_with_key(
    ciphertext: &[u8],
    key: &AeadKey,
    nonce: &[u8; 24],
    aad: &[u8],
) -> ObscuraResult<Vec<u8>> {
    AeadResult::decrypt(ciphertext, key, nonce, aad)
}
