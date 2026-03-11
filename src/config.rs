use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_project: Option<PathBuf>,
}

impl AppConfig {
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("failed to resolve home directory")?;
        Ok(home.join(".aipoor").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed reading config {}", path.display()))?;
        let config = toml::from_str(&content)
            .with_context(|| format!("failed parsing config {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed creating {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self).context("failed serializing config")?;
        fs::write(&path, content).with_context(|| format!("failed writing {}", path.display()))?;
        Ok(path)
    }

    pub fn resolve_project(&self, project: Option<&Path>) -> Option<PathBuf> {
        project
            .map(Path::to_path_buf)
            .or_else(|| self.default_project.clone())
    }
}
