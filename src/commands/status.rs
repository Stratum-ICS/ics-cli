use anyhow::Result;
use crate::commands::repo_root;
use crate::commit::{self, head_tree_or_empty};
use crate::paths;
use crate::store::Store;
use std::collections::BTreeSet;

pub fn cmd_status() -> Result<()> {
    let root = repo_root()?;
    let store = Store::open(&paths::ics_dir(&root))?;
    let head_tree = head_tree_or_empty(&store)?;
    let current = commit::working_tree_manifest(&root)?;

    let head_keys: BTreeSet<_> = head_tree.keys().cloned().collect();
    let cur_keys: BTreeSet<_> = current.keys().cloned().collect();

    if head_tree.is_empty() && current.is_empty() {
        println!("No tracked .md files.");
        return Ok(());
    }

    for k in head_keys.union(&cur_keys) {
        let old = head_tree.get(k);
        let new = current.get(k);
        let sym = match (old, new) {
            (None, Some(_)) => 'A',
            (Some(_), None) => 'D',
            (Some(a), Some(b)) if a != b => 'M',
            _ => continue,
        };
        println!("{sym} {k}");
    }
    Ok(())
}
