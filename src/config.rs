use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

const ENV_BASE_URL: &str = "STRATUM_BASE_URL";
const ENV_CONFIG_HOME: &str = "ICS_CONFIG_HOME";

fn strip_trailing_slash(mut s: String) -> String {
    while s.ends_with('/') {
        s.pop();
    }
    s
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub stratum_base_url: String,
    pub config_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let base = env::var(ENV_BASE_URL).unwrap_or_else(|_| "http://127.0.0.1:8000".into());
        let stratum_base_url = strip_trailing_slash(base);

        let config_dir = if let Ok(override_home) = env::var(ENV_CONFIG_HOME) {
            PathBuf::from(override_home)
        } else {
            let dirs = ProjectDirs::from("", "", "ics-cli").context("resolve project dirs")?;
            dirs.config_dir().to_path_buf()
        };

        Ok(Self {
            stratum_base_url,
            config_dir,
        })
    }

    pub fn for_tests(config_dir: PathBuf, base_url: String) -> Self {
        Self {
            stratum_base_url: strip_trailing_slash(base_url),
            config_dir,
        }
    }

    pub fn credentials_path(&self) -> PathBuf {
        self.config_dir.join("credentials.json")
    }
}
