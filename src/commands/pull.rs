use anyhow::{Context, Result};
use crate::auth_store;
use crate::commands::repo_root;
use crate::commit::worktree_matches_head;
use crate::config::AppConfig;
use crate::identity;
use crate::paths;
use crate::store::Store;
use crate::stratum::StratumClient;
use crate::sync::pull::{self, PullPolicy};
use crate::worktree;
use std::path::PathBuf;

pub async fn cmd_pull(
    all_tracked: bool,
    paths: Vec<PathBuf>,
    policy: PullPolicy,
    force: bool,
) -> Result<()> {
    let root = repo_root()?;
    let store = Store::open(&paths::ics_dir(&root))?;
    if !force && !worktree_matches_head(&root, &store)? {
        eprintln!(
            "error: working tree differs from HEAD (commit or checkout first, or pass --force)"
        );
        std::process::exit(2);
    }
    if !all_tracked && paths.is_empty() {
        anyhow::bail!("pull: specify PATH arguments or --all-tracked");
    }
    let cfg = AppConfig::from_env()?;
    let cred =
        auth_store::load_credentials(&cfg.credentials_path())?.context("run `ics login` first")?;
    let base = cred
        .base_url
        .clone()
        .unwrap_or_else(|| cfg.stratum_base_url.clone());
    let client = StratumClient::new(base, Some(cred.access_token.clone()));
    let index = identity::load_index(&root)?;
    let keys: Vec<String> = if all_tracked {
        let mut out = Vec::new();
        let mut missing = Vec::new();
        for rel in worktree::iter_tracked_md(&root)? {
            let key = worktree::posix_display(&rel);
            if index.contains_key(&key) {
                out.push(key);
            } else {
                missing.push(key);
            }
        }
        if !missing.is_empty() {
            eprintln!(
                "warning: {} tracked file(s) lack Stratum index entries (skipped): {}",
                missing.len(),
                missing.join(", ")
            );
        }
        out
    } else {
        let mut v = Vec::new();
        for p in paths {
            v.push(crate::path_safety::user_path_to_tree_key(&root, &p)?);
        }
        v
    };
    for key in keys {
        let entry = index
            .get(&key)
            .with_context(|| format!("no Stratum mapping for {key}; run `ics push` first"))?;
        let bytes = pull::fetch_note_body(&client, entry.note_id).await?;
        pull::apply_server_to_worktree(&root, &key, &bytes, policy)?;
    }
    Ok(())
}
