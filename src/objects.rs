use crate::error::{IcsError, Result};
use crate::paths;
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn blob_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn tree_hash(entries: &BTreeMap<String, String>) -> [u8; 32] {
    let bytes = serde_json::to_vec(entries).expect("BTreeMap serializes");
    blob_hash(&bytes)
}

pub fn write_blob(ics_dir: &Path, data: &[u8]) -> Result<[u8; 32]> {
    let hash = blob_hash(data);
    let path = paths::blob_path(ics_dir, &hash);
    if path.exists() {
        return Ok(hash);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, data)?;
    Ok(hash)
}

pub fn read_blob(ics_dir: &Path, hash: &[u8; 32]) -> Result<Vec<u8>> {
    let path = paths::blob_path(ics_dir, hash);
    fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IcsError::Msg(format!("missing blob {}", hex::encode(hash)))
        } else {
            IcsError::Io(e)
        }
    })
}

pub fn write_tree(ics_dir: &Path, entries: &BTreeMap<String, String>) -> Result<[u8; 32]> {
    let hash = tree_hash(entries);
    let path = paths::tree_path(ics_dir, &hash);
    if path.exists() {
        return Ok(hash);
    }
    let bytes = serde_json::to_vec(entries)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, bytes)?;
    Ok(hash)
}

pub fn read_tree(ics_dir: &Path, hash: &[u8; 32]) -> Result<BTreeMap<String, String>> {
    let path = paths::tree_path(ics_dir, hash);
    let bytes = fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IcsError::Msg(format!("missing tree {}", hex::encode(hash)))
        } else {
            IcsError::Io(e)
        }
    })?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn blob_hash_hello_known() {
        let h = blob_hash(b"hello");
        assert_eq!(
            hex::encode(h),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn blob_round_trip() {
        let dir = tempdir().unwrap();
        let ics = dir.path().join(".ics");
        fs::create_dir_all(ics.join("objects/blobs")).unwrap();
        let data = b"payload";
        let h = write_blob(&ics, data).unwrap();
        assert_eq!(read_blob(&ics, &h).unwrap(), data);
    }

    #[test]
    fn tree_stable_hash() {
        let mut m = BTreeMap::new();
        m.insert("a.md".into(), "aaa".into());
        m.insert("b.md".into(), "bbb".into());
        let h1 = tree_hash(&m);
        let h2 = tree_hash(&m);
        assert_eq!(h1, h2);
    }
}
