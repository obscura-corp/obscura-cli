use crate::util::errors::{ObscuraError, ObscuraResult};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::{rngs::OsRng, RngCore};
use zeroize::ZeroizeOnDrop;

const DEFAULT_MEMORY_KIB: u32 = 262_144;
const DEFAULT_TIME: u32 = 3;
const DEFAULT_LANES: u32 = 1;

#[derive(Clone, ZeroizeOnDrop)]
pub struct KdfParams {
    pub salt: [u8; 16],
    pub memory_kib: u32,
    pub time: u32,
    pub lanes: u32,
}

impl KdfParams {
    pub fn new() -> Self {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        Self {
            salt,
            memory_kib: DEFAULT_MEMORY_KIB,
            time: DEFAULT_TIME,
            lanes: DEFAULT_LANES,
        }
    }
}

#[derive(ZeroizeOnDrop)]
pub struct KdfResult {
    pub key: [u8; 32],
    pub params: KdfParams,
}

impl KdfResult {
    pub fn derive(passphrase: &str) -> ObscuraResult<Self> {
        let params = KdfParams::new();
        let key = derive_key(passphrase, &params)?;

        Ok(Self { key, params })
    }

    pub fn derive_with_params(passphrase: &str, params: &KdfParams) -> ObscuraResult<Self> {
        let key = derive_key(passphrase, params)?;

        Ok(Self {
            key,
            params: params.clone(),
        })
    }
}

fn derive_key(passphrase: &str, params: &KdfParams) -> ObscuraResult<[u8; 32]> {
    let argon2_params = Params::new(
        params.memory_kib,
        params.time,
        params.lanes,
        Some(Params::DEFAULT_OUTPUT_LEN),
    )
    .map_err(|_| ObscuraError::EncryptionFailed)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &params.salt, &mut key)
        .map_err(|_| ObscuraError::EncryptionFailed)?;

    Ok(key)
}
