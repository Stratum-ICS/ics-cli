use crate::error::IcsError;
use crate::path_safety;
use crate::stratum::{StratumClient, StratumError};
use clap::ValueEnum;
use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum PullPolicy {
    /// Keep local bytes when they differ from the server snapshot.
    #[value(name = "take-mine")]
    TakeMine,
    /// Overwrite local files with the server snapshot.
    #[value(name = "take-theirs")]
    TakeTheirs,
}

#[derive(Debug, Error)]
pub enum PullError {
    #[error("local version of {path} differs from server; choose --policy take-theirs or commit/reset first")]
    Conflict { path: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Stratum(#[from] StratumError),
    #[error(transparent)]
    Ics(#[from] IcsError),
}

pub async fn fetch_note_body(client: &StratumClient, note_id: u64) -> Result<Vec<u8>, StratumError> {
    let v = client.get_note(note_id).await?;
    let text = match v.get("body") {
        Some(Value::String(s)) => s.clone(),
        Some(other) => {
            return Err(StratumError::Msg(format!(
                "note {note_id} body is not a JSON string (got {other})"
            )));
        }
        None => {
            return Err(StratumError::Msg(format!(
                "note {note_id} response missing string field \"body\""
            )));
        }
    };
    Ok(text.into_bytes())
}

pub fn apply_server_to_worktree(
    repo_root: &Path,
    rel_key: &str,
    server: &[u8],
    policy: PullPolicy,
) -> Result<(), PullError> {
    path_safety::assert_safe_tree_key(rel_key)?;
    let dest: PathBuf = repo_root.join(Path::new(rel_key));
    let local = if dest.exists() {
        std::fs::read(&dest)?
    } else {
        Vec::new()
    };

    match policy {
        PullPolicy::TakeTheirs => {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, server)?;
        }
        PullPolicy::TakeMine => {
            if local.is_empty() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, server)?;
            } else if local != server {
                return Err(PullError::Conflict {
                    path: rel_key.into(),
                });
            }
        }
    }
    Ok(())
}
