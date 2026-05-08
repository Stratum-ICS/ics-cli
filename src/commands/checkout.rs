use anyhow::Result;
use crate::commands::repo_root;
use crate::commit::{read_blob_for_path, tree_at_head};
use crate::path_safety;
use crate::paths;
use crate::store::Store;
use std::fs;
use std::path::{Path, PathBuf};

pub fn cmd_checkout(all: bool, paths: Vec<PathBuf>) -> Result<()> {
    if !all && paths.is_empty() {
        anyhow::bail!(
            "checkout: specify PATH arguments or pass --all to restore the entire HEAD tree (destructive)"
        );
    }
    let root = repo_root()?;
    let store = Store::open(&paths::ics_dir(&root))?;
    let head_tree = tree_at_head(&store)?;
    let keys: Vec<String> = if all {
        head_tree.keys().cloned().collect()
    } else {
        let mut v = Vec::new();
        for p in &paths {
            v.push(path_safety::user_path_to_tree_key(&root, p)?);
        }
        v
    };
    for k in keys {
        let bytes = read_blob_for_path(&store, &head_tree, &k)?;
        let dest = root.join(Path::new(k.as_str()));
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, bytes)?;
    }
    Ok(())
}
