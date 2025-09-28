use crate::util::errors::{ObscuraError, ObscuraResult};
use rpassword::read_password;
use std::env;
use std::io::{self, Write};

pub fn prompt_passphrase() -> ObscuraResult<String> {
    print!("Enter vault passphrase (min 8 chars): ");
    io::stdout()
        .flush()
        .map_err(|_| ObscuraError::FilePermissionError)?;
    let passphrase = read_password().map_err(|_| ObscuraError::FilePermissionError)?;
    if passphrase.len() < 8 {
        return Err(ObscuraError::PassphraseTooShort);
    }
    Ok(passphrase)
}

pub fn prompt_passphrase_confirmation() -> ObscuraResult<String> {
    let passphrase = prompt_passphrase()?;
    print!("Confirm passphrase: ");
    io::stdout()
        .flush()
        .map_err(|_| ObscuraError::FilePermissionError)?;
    let confirmation = read_password().map_err(|_| ObscuraError::FilePermissionError)?;
    if passphrase != confirmation {
        return Err(ObscuraError::ConfirmationMismatch);
    }
    Ok(passphrase)
}

pub fn prompt_secret_value(alias: &str) -> ObscuraResult<String> {
    if let Ok(value) = env::var("OBSCURA_SECRET_VALUE") {
        return Ok(value);
    }
    print!("Enter value for '{}': ", alias);
    io::stdout()
        .flush()
        .map_err(|_| ObscuraError::FilePermissionError)?;
    read_password().map_err(|_| ObscuraError::FilePermissionError)
}

pub fn prompt_yes_no(message: &str) -> ObscuraResult<bool> {
    print!("{} (y/N): ", message);
    io::stdout()
        .flush()
        .map_err(|_| ObscuraError::FilePermissionError)?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| ObscuraError::FilePermissionError)?;
    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

pub fn get_passphrase_from_env() -> Option<String> {
    std::env::var("OBSCURA_PASSPHRASE").ok()
}
