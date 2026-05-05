use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

pub fn load_credentials(path: &Path) -> Result<Option<Credentials>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let c: Credentials = serde_json::from_str(&raw).context("parse credentials.json")?;
    Ok(Some(c))
}

pub fn save_credentials(path: &Path, cred: &Credentials) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(cred).context("serialize credentials")?;
    atomic_write_json(path, &data).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn atomic_write_json(path: &Path, data: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("credentials.json"));
    let tmp_path = dir.join(format!(
        ".{}.tmp.{}",
        name.to_string_lossy(),
        std::process::id()
    ));
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(data.as_bytes())?;
        f.sync_all()?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&tmp_path, perms)?;
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip_credentials() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("credentials.json");
        let c = Credentials {
            access_token: "tok".into(),
            base_url: Some("http://localhost".into()),
        };
        save_credentials(&p, &c).unwrap();
        let loaded = load_credentials(&p).unwrap().unwrap();
        assert_eq!(loaded.access_token, "tok");
    }
}
