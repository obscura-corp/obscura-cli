use std::fmt;

#[derive(Debug, Clone)]
pub enum ObscuraError {
    VaultNotFound,
    AliasNotFound(String),
    DecryptionFailed,
    EncryptionFailed,
    FilePermissionError,
    InvalidVaultFormat,
    PassphraseTooShort,
    ConfirmationMismatch,
    FileExists(String),
    InvalidTimeout,
    CustomError(String),
}

impl fmt::Display for ObscuraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObscuraError::VaultNotFound => write!(f, "Vault not found"),
            ObscuraError::AliasNotFound(alias) => write!(f, "Alias '{}' not found", alias),
            ObscuraError::DecryptionFailed => write!(f, "Decryption failed, re-enter passphrase or try 'obscura lock' to refresh the vault"),
            ObscuraError::EncryptionFailed => write!(f, "Encryption failed"),
            ObscuraError::FilePermissionError => write!(f, "File permission error"),
            ObscuraError::InvalidVaultFormat => write!(f, "Invalid vault format"),
            ObscuraError::PassphraseTooShort => {
                write!(f, "Passphrase must be at least 8 characters")
            }
            ObscuraError::ConfirmationMismatch => {
                write!(f, "Passphrase confirmation does not match")
            }
            ObscuraError::FileExists(path) => write!(f, "File '{}' already exists", path),
            ObscuraError::InvalidTimeout => write!(f, "Invalid timeout value"),
            ObscuraError::CustomError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ObscuraError {}

impl From<std::io::Error> for ObscuraError {
    fn from(_: std::io::Error) -> Self {
        ObscuraError::FilePermissionError
    }
}

impl From<serde_json::Error> for ObscuraError {
    fn from(_: serde_json::Error) -> Self {
        ObscuraError::InvalidVaultFormat
    }
}

pub type ObscuraResult<T> = Result<T, ObscuraError>;
