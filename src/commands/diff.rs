use anyhow::Result;
use crate::commands::repo_root;
use crate::commit::{head_tree_or_empty, read_blob_for_path, working_tree_manifest};
use crate::path_safety;
use crate::paths;
use crate::store::Store;
use similar::TextDiff;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn cmd_diff(paths: Vec<PathBuf>) -> Result<()> {
    let root = repo_root()?;
    let store = Store::open(&paths::ics_dir(&root))?;
    let head_tree = head_tree_or_empty(&store)?;
    let current = working_tree_manifest(&root)?;

    let filter: Option<BTreeSet<String>> = if paths.is_empty() {
        None
    } else {
        let mut s = BTreeSet::new();
        for p in &paths {
            s.insert(path_safety::user_path_to_tree_key(&root, p)?);
        }
        Some(s)
    };

    let keys: Vec<String> = head_tree
        .keys()
        .chain(current.keys())
        .filter(|k| filter.as_ref().map(|f| f.contains(*k)).unwrap_or(true))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    for k in keys {
        let old_h = head_tree.get(&k);
        let new_h = current.get(&k);
        if old_h == new_h {
            continue;
        }
        let old_text = match old_h {
            Some(_) => {
                let bytes = read_blob_for_path(&store, &head_tree, &k)?;
                String::from_utf8_lossy(&bytes).into_owned()
            }
            None => String::new(),
        };
        let new_text = match new_h {
            Some(_) => {
                let bytes = fs::read(root.join(Path::new(k.as_str())))?;
                String::from_utf8_lossy(&bytes).into_owned()
            }
            None => String::new(),
        };
        let diff = TextDiff::from_lines(&old_text, &new_text);
        let header_a = format!("a/{k}");
        let header_b = format!("b/{k}");
        print!("{}", diff.unified_diff().header(&header_a, &header_b));
    }
    Ok(())
}
