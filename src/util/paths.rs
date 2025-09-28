use crate::util::errors::{ObscuraError, ObscuraResult};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

pub fn get_config_dir() -> ObscuraResult<PathBuf> {
    let project_dirs =
        ProjectDirs::from("", "", "Obscura").ok_or(ObscuraError::FilePermissionError)?;

    Ok(project_dirs.config_dir().to_path_buf())
}

pub fn get_global_vault_path() -> ObscuraResult<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("vault.enc"))
}

pub fn get_projects_dir() -> ObscuraResult<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("projects"))
}

pub fn get_project_vault_path(project_path: &Path) -> ObscuraResult<PathBuf> {
    let canonical_path = project_path
        .canonicalize()
        .map_err(|_| ObscuraError::FilePermissionError)?;

    let hash = blake3::hash(canonical_path.to_string_lossy().as_bytes());
    let projects_dir = get_projects_dir()?;
    Ok(projects_dir
        .join(hex::encode(hash.as_bytes()))
        .join("vault.enc"))
}

pub fn get_project_meta_path(project_path: &Path) -> ObscuraResult<PathBuf> {
    let canonical_path = project_path
        .canonicalize()
        .map_err(|_| ObscuraError::FilePermissionError)?;

    let hash = blake3::hash(canonical_path.to_string_lossy().as_bytes());
    let projects_dir = get_projects_dir()?;
    Ok(projects_dir
        .join(hex::encode(hash.as_bytes()))
        .join("meta.json"))
}

pub fn get_agent_socket_path() -> ObscuraResult<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("agent.sock"))
}

pub fn ensure_config_dir() -> ObscuraResult<()> {
    let config_dir = get_config_dir()?;
    std::fs::create_dir_all(&config_dir).map_err(|_| ObscuraError::FilePermissionError)?;
    Ok(())
}

pub fn ensure_projects_dir() -> ObscuraResult<()> {
    let projects_dir = get_projects_dir()?;
    std::fs::create_dir_all(&projects_dir).map_err(|_| ObscuraError::FilePermissionError)?;
    Ok(())
}
