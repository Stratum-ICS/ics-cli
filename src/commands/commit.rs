use anyhow::Result;
use crate::commands::repo_root;
use crate::commit::{self, CommitOptions};
use crate::error::IcsError;
use crate::paths;
use crate::store::Store;

fn default_author() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

pub fn cmd_commit(message: String, author: Option<String>) -> Result<()> {
    let root = repo_root()?;
    let store = Store::open(&paths::ics_dir(&root))?;
    let author = author.unwrap_or_else(default_author);
    let id = commit::make_commit(
        &store,
        &root,
        CommitOptions {
            message: &message,
            author: &author,
        },
    )
    .map_err(|e| match e {
        IcsError::NothingToCommit => anyhow::anyhow!("nothing to commit, working tree clean"),
        _ => e.into(),
    })?;
    println!("{id}");
    Ok(())
}
