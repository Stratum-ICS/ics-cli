use crate::error::{IcsError, Result};
use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexEntry {
    pub owner_user_id: u64,
    pub note_id: u64,
}

pub type StratumIndex = BTreeMap<String, IndexEntry>;

pub fn index_path(repo_root: &Path) -> PathBuf {
    paths::ics_dir(repo_root).join("stratum-index.json")
}

pub fn load_index(repo_root: &Path) -> Result<StratumIndex> {
    let p = index_path(repo_root);
    if !p.exists() {
        return Ok(BTreeMap::new());
    }
    let raw = fs::read_to_string(&p).map_err(IcsError::from)?;
    serde_json::from_str(&raw).map_err(|e| IcsError::Msg(e.to_string()))
}

pub fn save_index(repo_root: &Path, index: &StratumIndex) -> Result<()> {
    let p = index_path(repo_root);
    let data =
        serde_json::to_string_pretty(index).map_err(|e| IcsError::Msg(e.to_string()))?;
    fs::write(&p, data).map_err(IcsError::from)?;
    Ok(())
}

fn yaml_u64(m: &serde_yaml::Mapping, key: &str) -> Option<u64> {
    m.iter()
        .find(|(k, _)| k.as_str() == Some(key))
        .and_then(|(_, v)| v.as_u64())
}

pub fn resolve_ids(
    index: &StratumIndex,
    tree_key: &str,
    fm: &serde_yaml::Mapping,
) -> Option<IndexEntry> {
    let note = yaml_u64(fm, "stratum_note_id");
    let owner = yaml_u64(fm, "stratum_owner_id");
    if let (Some(note_id), Some(owner_user_id)) = (note, owner) {
        return Some(IndexEntry {
            owner_user_id,
            note_id,
        });
    }
    index.get(tree_key).cloned()
}
