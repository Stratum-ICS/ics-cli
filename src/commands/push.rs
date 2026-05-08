use anyhow::{Context, Result};
use crate::auth_store;
use crate::commands::repo_root;
use crate::config::AppConfig;
use crate::frontmatter::{self, merge_frontmatter};
use crate::identity::{self};
use crate::stratum::StratumClient;
use crate::worktree;
use serde_json::json;
use std::fs;
use std::path::Path;

fn tracked_tree_keys(repo_root: &Path) -> Result<Vec<String>> {
    let mut v = Vec::new();
    for rel in worktree::iter_tracked_md(repo_root)? {
        v.push(worktree::posix_display(&rel));
    }
    v.sort();
    Ok(v)
}

async fn push_one(
    root: &Path,
    client: &StratumClient,
    index: &mut identity::StratumIndex,
    key: &str,
) -> Result<()> {
    let path = root.join(Path::new(key));
    let raw = fs::read_to_string(&path)?;
    let (map_opt, doc_body) = frontmatter::parse_frontmatter_map(&raw)?;
    let fm_map = map_opt.unwrap_or_default();
    let entry = identity::resolve_ids(index, key, &fm_map);
    let fm_has_note_id = fm_map.keys().any(|k| k.as_str() == Some("stratum_note_id"));
    if entry.is_some() && !fm_has_note_id {
        eprintln!("warning: {} lacks frontmatter; using cached index entry", key);
    }
    let body_json = json!({
        "title": key,
        "body": doc_body,
    });
    let resp = if let Some(e) = entry {
        client.update_note(e.note_id, &body_json).await?
    } else {
        client.create_note(&body_json).await?
    };
    let note_id = resp
        .get("id")
        .and_then(|x| x.as_u64())
        .context("missing note id in response")?;
    let owner = resp
        .get("owner_user_id")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let mut patch = serde_yaml::Mapping::new();
    patch.insert(
        serde_yaml::Value::String("stratum_note_id".into()),
        serde_yaml::Value::Number(note_id.into()),
    );
    patch.insert(
        serde_yaml::Value::String("stratum_owner_id".into()),
        serde_yaml::Value::Number(owner.into()),
    );
    let updated = merge_frontmatter(&raw, patch)?;
    fs::write(&path, updated)?;
    // NOTE: do NOT write to stratum-index.json — frontmatter is the sole
    // source of truth for note IDs. The index is a read-only cache only.
    Ok(())
}

pub async fn cmd_push(paths: Vec<std::path::PathBuf>) -> Result<()> {
    let root = repo_root()?;
    let cfg = AppConfig::from_env()?;
    let cred =
        auth_store::load_credentials(&cfg.credentials_path())?.context("run `ics login` first")?;
    let base = cred
        .base_url
        .clone()
        .unwrap_or_else(|| cfg.stratum_base_url.clone());
    let client = StratumClient::new(base, Some(cred.access_token.clone()));
    let mut index = identity::load_index(&root)?;
    let keys: Vec<String> = if paths.is_empty() {
        tracked_tree_keys(&root)?
    } else {
        let mut v = Vec::new();
        for p in paths {
            v.push(crate::path_safety::user_path_to_tree_key(&root, &p)?);
        }
        v
    };
    for key in keys {
        push_one(&root, &client, &mut index, &key).await?;
    }
    identity::save_index(&root, &index)?;
    Ok(())
}
