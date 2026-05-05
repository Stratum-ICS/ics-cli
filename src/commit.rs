use crate::error::{IcsError, Result};
use crate::objects;
use crate::path_safety;
use crate::store::Store;
use crate::worktree;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub struct CommitOptions<'a> {
    pub message: &'a str,
    pub author: &'a str,
}

fn decode_hex32(s: &str) -> Result<[u8; 32]> {
    let v = hex::decode(s)?;
    if v.len() != 32 {
        return Err(IcsError::Msg("expected 32-byte hash".into()));
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    Ok(a)
}

pub fn working_tree_manifest(repo_root: &Path) -> Result<BTreeMap<String, String>> {
    let rel_paths = worktree::iter_tracked_md(repo_root)?;
    let mut tree_map = BTreeMap::new();
    for rel in rel_paths {
        let key = worktree::posix_display(&rel);
        path_safety::assert_safe_tree_key(&key)?;
        let data = fs::read(repo_root.join(&rel))?;
        tree_map.insert(key, hex::encode(objects::blob_hash(&data)));
    }
    Ok(tree_map)
}

pub fn tree_at_head(store: &Store) -> Result<BTreeMap<String, String>> {
    let head = store.head_commit()?.ok_or(IcsError::NoCommits)?;
    let row = store
        .get_commit_row(&head)?
        .ok_or_else(|| IcsError::Msg("HEAD points to missing commit".into()))?;
    let tid = decode_hex32(&row.tree_id)?;
    let tree = objects::read_tree(store.ics_dir.as_path(), &tid)?;
    for k in tree.keys() {
        path_safety::assert_safe_tree_key(k)?;
    }
    Ok(tree)
}

/// HEAD snapshot tree, or empty map when there are no commits yet.
pub fn head_tree_or_empty(store: &Store) -> Result<BTreeMap<String, String>> {
    match tree_at_head(store) {
        Ok(t) => Ok(t),
        Err(IcsError::NoCommits) => Ok(BTreeMap::new()),
        Err(e) => Err(e),
    }
}

pub fn make_commit(store: &Store, repo_root: &Path, opts: CommitOptions<'_>) -> Result<String> {
    let tree_map = working_tree_manifest(repo_root)?;

    if let Some(cid) = store.head_commit()? {
        let row = store
            .get_commit_row(&cid)?
            .ok_or_else(|| IcsError::Msg("HEAD points to missing commit".into()))?;
        let tid = decode_hex32(&row.tree_id)?;
        let head_tree = objects::read_tree(store.ics_dir.as_path(), &tid)?;
        for k in head_tree.keys() {
            path_safety::assert_safe_tree_key(k)?;
        }
        if head_tree == tree_map {
            return Err(IcsError::NothingToCommit);
        }
    }

    for rel in worktree::iter_tracked_md(repo_root)? {
        let full = repo_root.join(&rel);
        let data = fs::read(&full)?;
        objects::write_blob(store.ics_dir.as_path(), &data)?;
    }

    let tree_id_hex = hex::encode(objects::write_tree(store.ics_dir.as_path(), &tree_map)?);

    let parents: Vec<String> = match store.head_commit()? {
        Some(cid) => vec![cid],
        None => vec![],
    };
    let parents_json = serde_json::to_string(&parents)?;

    let created_at = unix_now();

    let mut commit_map: BTreeMap<&str, Value> = BTreeMap::new();
    commit_map.insert("author", Value::String(opts.author.to_string()));
    commit_map.insert("created_at", Value::Number(created_at.into()));
    commit_map.insert("message", Value::String(opts.message.to_string()));
    commit_map.insert("parents", serde_json::from_str(&parents_json)?);
    commit_map.insert("tree_id", Value::String(tree_id_hex.clone()));

    let commit_bytes = serde_json::to_vec(&commit_map)?;
    let commit_hash: [u8; 32] = {
        let mut h = Sha256::new();
        h.update(&commit_bytes);
        h.finalize().into()
    };
    let commit_id = hex::encode(commit_hash);

    store.insert_commit(
        &commit_id,
        &tree_id_hex,
        &parents_json,
        opts.message,
        opts.author,
        created_at,
    )?;
    store.set_ref("HEAD", &commit_id)?;
    store.set_ref("main", &commit_id)?;

    Ok(commit_id)
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn read_blob_for_path(
    store: &Store,
    head_tree: &BTreeMap<String, String>,
    rel_key: &str,
) -> Result<Vec<u8>> {
    path_safety::assert_safe_tree_key(rel_key)?;
    let blob_hex = head_tree
        .get(rel_key)
        .ok_or_else(|| IcsError::Msg(format!("path not in HEAD: {rel_key}")))?;
    let hash = decode_hex32(blob_hex)?;
    objects::read_blob(store.ics_dir.as_path(), &hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths;
    use crate::store::Store;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn init_commit_sets_head() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        Store::init(root).unwrap();
        fs::write(root.join("a.md"), "hello").unwrap();
        let store = Store::open(&paths::ics_dir(root)).unwrap();
        let id = make_commit(
            &store,
            root,
            CommitOptions {
                message: "first",
                author: "t",
            },
        )
        .unwrap();
        assert_eq!(store.head_commit().unwrap(), Some(id.clone()));
    }
}
