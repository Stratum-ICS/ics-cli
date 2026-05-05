use std::path::{Path, PathBuf};

pub fn ics_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".ics")
}

pub fn store_db_path(ics_dir: &Path) -> PathBuf {
    ics_dir.join("store.db")
}

pub fn blob_path(ics_dir: &Path, hash: &[u8; 32]) -> PathBuf {
    let hex = hex::encode(hash);
    ics_dir.join("objects/blobs").join(&hex[..2]).join(&hex[2..])
}

pub fn tree_path(ics_dir: &Path, hash: &[u8; 32]) -> PathBuf {
    let hex = hex::encode(hash);
    ics_dir.join("objects/trees").join(&hex[..2]).join(&hex[2..])
}
