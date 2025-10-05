use crate::util::errors::{ObscuraError, ObscuraResult};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::{rngs::OsRng, RngCore};
use std::env;
use std::sync::OnceLock;
use zeroize::ZeroizeOnDrop;

const DEFAULT_MEMORY_KIB: u32 = 131_072;
const DEFAULT_TIME: u32 = 2;
const DEFAULT_LANES: u32 = 1;
const MIN_MEMORY_KIB: u32 = 65_536;
const MAX_MEMORY_KIB: u32 = 524_288;
const MIN_TIME: u32 = 1;
const MAX_TIME: u32 = 6;

static KDF_PARAMS_CACHE: OnceLock<(u32, u32)> = OnceLock::new();

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

        let (memory_kib, time) = get_cached_kdf_params();
        Self {
            salt,
            memory_kib,
            time,
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
    .or_else(|_| {
        Params::new(
            MIN_MEMORY_KIB,
            params.time,
            params.lanes,
            Some(Params::DEFAULT_OUTPUT_LEN),
        )
    })
    .map_err(|_| ObscuraError::EncryptionFailed)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &params.salt, &mut key)
        .map_err(|_| ObscuraError::EncryptionFailed)?;

    Ok(key)
}

fn get_cached_kdf_params() -> (u32, u32) {
    *KDF_PARAMS_CACHE.get_or_init(|| {
        let memory_kib = env_value("OBSCURA_KDF_MEM_KIB")
            .map(|value| value.clamp(MIN_MEMORY_KIB, MAX_MEMORY_KIB))
            .unwrap_or(DEFAULT_MEMORY_KIB);
        let time = env_value("OBSCURA_KDF_TIME")
            .map(|value| value.clamp(MIN_TIME, MAX_TIME))
            .unwrap_or(DEFAULT_TIME);
        (memory_kib, time)
    })
}

fn env_value(name: &str) -> Option<u32> {
    env::var(name).ok()?.parse().ok()
}
