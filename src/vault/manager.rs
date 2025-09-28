use crate::util::errors::{ObscuraError, ObscuraResult};
use crate::util::paths::{
    ensure_config_dir, ensure_projects_dir, get_global_vault_path, get_project_meta_path,
    get_project_vault_path,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultType {
    Global,
    Project,
}

#[derive(Debug, Clone)]
pub struct VaultInfo {
    pub vault_type: VaultType,
    pub path: PathBuf,
}

impl VaultInfo {
    pub fn global() -> ObscuraResult<Self> {
        Ok(Self {
            vault_type: VaultType::Global,
            path: get_global_vault_path()?,
        })
    }

    pub fn project(project_path: &Path) -> ObscuraResult<Self> {
        Ok(Self {
            vault_type: VaultType::Project,
            path: get_project_vault_path(project_path)?,
        })
    }
}

pub struct VaultManager;

impl VaultManager {
    pub fn resolve_vault(force_global: bool, force_project: bool) -> ObscuraResult<VaultInfo> {
        if force_global && force_project {
            return Err(ObscuraError::InvalidVaultFormat);
        }

        if force_global {
            return VaultInfo::global();
        }

        if force_project {
            let current_dir =
                std::env::current_dir().map_err(|_| ObscuraError::FilePermissionError)?;
            return VaultInfo::project(&current_dir);
        }

        let current_dir = std::env::current_dir().map_err(|_| ObscuraError::FilePermissionError)?;

        let project_vault = VaultInfo::project(&current_dir)?;
        if project_vault.path.exists() {
            return Ok(project_vault);
        }

        VaultInfo::global()
    }

    pub fn ensure_global_vault() -> ObscuraResult<()> {
        ensure_config_dir()?;
        Ok(())
    }

    pub fn ensure_project_vault(project_path: &Path) -> ObscuraResult<()> {
        ensure_projects_dir()?;
        let vault_path = get_project_vault_path(project_path)?;
        if let Some(parent) = vault_path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| ObscuraError::FilePermissionError)?;
        }
        let meta_path = get_project_meta_path(project_path)?;
        let meta = ProjectMeta {
            path: project_path
                .canonicalize()
                .map_err(|_| ObscuraError::FilePermissionError)?
                .to_string_lossy()
                .to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: chrono::Utc::now().to_rfc3339(),
        };
        let meta_json =
            serde_json::to_string_pretty(&meta).map_err(|_| ObscuraError::FilePermissionError)?;
        std::fs::write(&meta_path, meta_json).map_err(|_| ObscuraError::FilePermissionError)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectMeta {
    path: String,
    created_at: String,
    last_used: String,
}
