use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ics_cli::auth_store::{self, Credentials};
use ics_cli::commit::{self, CommitOptions};
use ics_cli::config::AppConfig;
use ics_cli::error::IcsError;
use ics_cli::frontmatter::{self, merge_frontmatter};
use ics_cli::identity::{self, IndexEntry};
use ics_cli::path_safety;
use ics_cli::paths;
use ics_cli::repo;
use ics_cli::store::Store;
use ics_cli::stratum::routes;
use ics_cli::stratum::vault_client::{vault_pull_apply, vault_pull_preview};
use ics_cli::stratum::StratumClient;
use ics_cli::sync::pull::{self, PullPolicy};
use ics_cli::worktree;
use serde_json::{json, Value};
use similar::TextDiff;
use std::collections::BTreeSet;
use std::fs;
use std::io::{stdin, IsTerminal};
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
        #[arg(long)]
        all: bool,
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// Authenticate against Stratum and store credentials locally (`0600`).
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: Option<String>,
    },
    /// Create or update Stratum notes from markdown files (YAML frontmatter ids).
    Push {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    Proposal {
        #[command(subcommand)]
        sub: ProposalCmd,
    },
    /// Fetch note bodies from Stratum into the worktree (every tracked `.md` that has an index entry).
    Pull {
        #[arg(long)]
        all_tracked: bool,
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
        #[arg(long)]
        policy: PullPolicy,
        #[arg(long)]
        force: bool,
    },
    Vault {
        #[command(subcommand)]
        sub: VaultCmd,
    },
}

#[derive(Subcommand)]
enum ProposalCmd {
    /// Submit a proposal (`POST /api/proposals`). Rationale must be at least 50 characters.
    Submit {
        #[arg(long)]
        team_id: u64,
        #[arg(long, value_delimiter = ',')]
        note_ids: Vec<u64>,
        #[arg(long)]
        rationale: String,
    },
}

#[derive(Subcommand)]
enum VaultCmd {
    /// Preview then optionally apply `POST /api/vaults/{slug}/pull`.
    Pull {
        slug: String,
        #[arg(long, help = "Apply server changes without prompting (destructive)")]
        yes: bool,
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

fn extract_access_token(v: &Value) -> Option<String> {
    v.get("access_token")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            v.get("token")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
        })
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

async fn cmd_login(username: String, password: Option<String>) -> Result<()> {
    let cfg = AppConfig::from_env()?;
    let password = password
        .or_else(|| std::env::var("STRATUM_PASSWORD").ok())
        .context("password required (--password or STRATUM_PASSWORD)")?;
    let client = StratumClient::new(cfg.stratum_base_url.clone(), None);
    let v = client.login(&username, &password).await?;
    let token = extract_access_token(&v).context("login response missing access_token/token")?;
    let cred = Credentials {
        access_token: token,
        base_url: Some(cfg.stratum_base_url.clone()),
    };
    auth_store::save_credentials(&cfg.credentials_path(), &cred)?;
    println!("saved credentials to {}", cfg.credentials_path().display());
    Ok(())
}

fn tracked_tree_keys(repo_root: &Path) -> Result<Vec<String>> {
    let mut v = Vec::new();
    for rel in worktree::iter_tracked_md(repo_root)? {
        v.push(worktree::posix_display(&rel));
    }
    v.sort();
    Ok(v)
}

async fn cmd_push(paths: Vec<PathBuf>) -> Result<()> {
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
            v.push(path_safety::user_path_to_tree_key(&root, &p)?);
        }
        v
    };
    for key in keys {
        push_one(&root, &client, &mut index, &key).await?;
    }
    identity::save_index(&root, &index)?;
    Ok(())
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
    index.insert(
        key.to_string(),
        IndexEntry {
            owner_user_id: owner,
            note_id,
        },
    );
    Ok(())
}

async fn cmd_proposal_submit(team_id: u64, note_ids: Vec<u64>, rationale: String) -> Result<()> {
    if rationale.chars().count() < 50 {
        anyhow::bail!("rationale must be at least 50 characters for proposals");
    }
    let cfg = AppConfig::from_env()?;
    let cred =
        auth_store::load_credentials(&cfg.credentials_path())?.context("run `ics login` first")?;
    let base = cred
        .base_url
        .clone()
        .unwrap_or_else(|| cfg.stratum_base_url.clone());
    let client = StratumClient::new(base, Some(cred.access_token.clone()));
    let body = json!({
        "team_id": team_id,
        "note_ids": note_ids,
        "rationale": rationale,
    });
    let resp = client.post_proposal(&body).await;
    match resp {
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v)?),
        Err(e) => {
            anyhow::bail!("{}", e);
        }
    }
    Ok(())
}

async fn cmd_pull(
    all_tracked: bool,
    paths: Vec<PathBuf>,
    policy: PullPolicy,
    force: bool,
) -> Result<()> {
    let root = repo_root()?;
    let store = Store::open(&paths::ics_dir(&root))?;
    if !force && !commit::worktree_matches_head(&root, &store)? {
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
            v.push(path_safety::user_path_to_tree_key(&root, &p)?);
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

async fn cmd_vault_pull(slug: String, yes: bool) -> Result<()> {
    routes::assert_safe_vault_slug(&slug).map_err(|e| anyhow::anyhow!("{}", e))?;
    let cfg = AppConfig::from_env()?;
    let cred =
        auth_store::load_credentials(&cfg.credentials_path())?.context("run `ics login` first")?;
    let base = cred
        .base_url
        .clone()
        .unwrap_or_else(|| cfg.stratum_base_url.clone());
    let client = StratumClient::new(base, Some(cred.access_token.clone()));
    let preview = vault_pull_preview(&client, &slug, json!({})).await?;
    println!("{}", serde_json::to_string_pretty(&preview)?);
    if !yes && !stdin().is_terminal() {
        anyhow::bail!("vault pull: non-interactive mode requires --yes");
    }
    let apply = if yes {
        true
    } else {
        dialoguer::Confirm::new()
            .with_prompt("Apply vault pull with confirm=true?")
            .default(false)
            .interact()?
    };
    if !apply {
        println!("aborted");
        return Ok(());
    }
    let apply_body = if preview.is_object() {
        preview
    } else {
        anyhow::bail!(
            "vault preview response must be a JSON object to apply; refusing empty confirm=true body"
        );
    };
    let applied = vault_pull_apply(&client, &slug, apply_body).await?;
    println!("{}", serde_json::to_string_pretty(&applied)?);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
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
                IcsError::NothingToCommit => {
                    anyhow::anyhow!("nothing to commit, working tree clean")
                }
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
        Cmd::Login { username, password } => {
            cmd_login(username, password).await?;
        }
        Cmd::Push { paths } => {
            cmd_push(paths).await?;
        }
        Cmd::Proposal { sub } => match sub {
            ProposalCmd::Submit {
                team_id,
                note_ids,
                rationale,
            } => {
                cmd_proposal_submit(team_id, note_ids, rationale).await?;
            }
        },
        Cmd::Pull {
            all_tracked,
            paths,
            policy,
            force,
        } => {
            cmd_pull(all_tracked, paths, policy, force).await?;
        }
        Cmd::Vault { sub } => match sub {
            VaultCmd::Pull { slug, yes } => {
                cmd_vault_pull(slug, yes).await?;
            }
        },
    }
    Ok(())
}
