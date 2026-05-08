use anyhow::Result;
use crate::error::IcsError;
use crate::store::Store;
use std::path::PathBuf;

pub fn cmd_init(path: PathBuf) -> Result<()> {
    let root = crate::repo::resolve_repo_root(Some(path))?;
    Store::init(&root).map_err(|e| match e {
        IcsError::AlreadyInitialized => anyhow::anyhow!("already an ics repository"),
        _ => e.into(),
    })?;
    println!("Initialized empty ics repository in {}", root.display());
    Ok(())
}
