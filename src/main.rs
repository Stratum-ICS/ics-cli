use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ics_cli::commit::{self, CommitOptions};
use ics_cli::error::IcsError;
use ics_cli::path_safety;
use ics_cli::paths;
use ics_cli::repo;
use ics_cli::store::Store;
use similar::TextDiff;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ics", version, about = "Local markdown workspace with git-like history")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create an empty `.ics/` repository in the given directory (default: current).
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show changed markdown paths vs HEAD (or all new if no commits).
    Status,
    /// Record a snapshot of tracked `*.md` files (excluding `.ics/`).
    Commit {
        #[arg(short = 'm', long)]
        message: String,
        #[arg(long)]
        author: Option<String>,
    },
    /// Print commits starting from HEAD (newest first).
    Log,
    /// Unified diff vs HEAD for changed paths (default: all changes).
    Diff {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// Overwrite working-tree files from the HEAD tree (destructive).
    Checkout {
        /// Restore every path in the HEAD snapshot (overwrites local files).
        #[arg(long)]
        all: bool,
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
}

fn default_author() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn repo_root() -> Result<PathBuf> {
    repo::find_repo_from_env()
        .ok_or(IcsError::NotRepository)
        .map_err(|e| e.into())
}

fn cmd_status(repo_root: &Path) -> Result<()> {
    let store = Store::open(&paths::ics_dir(repo_root))?;
    let head_tree = commit::head_tree_or_empty(&store)?;
    let current = commit::working_tree_manifest(repo_root)?;

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

fn cmd_diff(repo_root: &Path, path_args: &[PathBuf]) -> Result<()> {
    let store = Store::open(&paths::ics_dir(repo_root))?;
    let head_tree = commit::head_tree_or_empty(&store)?;
    let current = commit::working_tree_manifest(repo_root)?;

    let filter: Option<BTreeSet<String>> = if path_args.is_empty() {
        None
    } else {
        let mut s = BTreeSet::new();
        for p in path_args {
            s.insert(path_safety::user_path_to_tree_key(repo_root, p)?);
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
                let bytes = commit::read_blob_for_path(&store, &head_tree, &k)?;
                String::from_utf8_lossy(&bytes).into_owned()
            }
            None => String::new(),
        };
        let new_text = match new_h {
            Some(_) => {
                let bytes = fs::read(repo_root.join(Path::new(k.as_str())))?;
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

fn cmd_log(repo_root: &Path) -> Result<()> {
    let store = Store::open(&paths::ics_dir(repo_root))?;
    let mut id = match store.head_commit()? {
        Some(c) => c,
        None => {
            println!("No commits yet.");
            return Ok(());
        }
    };
    loop {
        let row = store
            .get_commit_row(&id)?
            .with_context(|| format!("missing commit row for {id}"))?;
        println!(
            "commit {}\nAuthor: {}\nDate:   {}\n\n    {}\n",
            row.commit_id, row.author, row.created_at, row.message
        );
        let parents: Vec<String> = serde_json::from_str(&row.parent_commit_ids)?;
        if parents.is_empty() {
            break;
        }
        id = parents[0].clone();
    }
    Ok(())
}

fn cmd_checkout(repo_root: &Path, all: bool, path_args: &[PathBuf]) -> Result<()> {
    if !all && path_args.is_empty() {
        anyhow::bail!(
            "checkout: specify PATH arguments or pass --all to restore the entire HEAD tree (destructive)"
        );
    }
    let store = Store::open(&paths::ics_dir(repo_root))?;
    let head_tree = commit::tree_at_head(&store)?;
    let keys: Vec<String> = if all {
        head_tree.keys().cloned().collect()
    } else {
        let mut v = Vec::new();
        for p in path_args {
            v.push(path_safety::user_path_to_tree_key(repo_root, p)?);
        }
        v
    };
    for k in keys {
        let bytes = commit::read_blob_for_path(&store, &head_tree, &k)?;
        let dest = repo_root.join(Path::new(k.as_str()));
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, bytes)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { path } => {
            let root = repo::resolve_repo_root(Some(path))?;
            Store::init(&root).map_err(|e| match e {
                IcsError::AlreadyInitialized => anyhow::anyhow!("already an ics repository"),
                _ => e.into(),
            })?;
            println!("Initialized empty ics repository in {}", root.display());
        }
        Cmd::Status => {
            let root = repo_root()?;
            cmd_status(&root)?;
        }
        Cmd::Commit { message, author } => {
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
        }
        Cmd::Log => {
            let root = repo_root()?;
            cmd_log(&root)?;
        }
        Cmd::Diff { paths } => {
            let root = repo_root()?;
            cmd_diff(&root, &paths)?;
        }
        Cmd::Checkout { all, paths } => {
            let root = repo_root()?;
            cmd_checkout(&root, all, &paths)?;
        }
    }
    Ok(())
}
