use anyhow::Result;
use clap::{Parser, Subcommand};
use ics_cli::sync::pull::PullPolicy;

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
        path: std::path::PathBuf,
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
        paths: Vec<std::path::PathBuf>,
    },
    /// Overwrite working-tree files from the HEAD tree (destructive).
    Checkout {
        #[arg(long)]
        all: bool,
        #[arg(value_name = "PATH")]
        paths: Vec<std::path::PathBuf>,
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
        paths: Vec<std::path::PathBuf>,
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
        paths: Vec<std::path::PathBuf>,
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { path } => {
            ics_cli::commands::init::cmd_init(path)?;
        }
        Cmd::Status => {
            ics_cli::commands::status::cmd_status()?;
        }
        Cmd::Commit { message, author } => {
            ics_cli::commands::commit::cmd_commit(message, author)?;
        }
        Cmd::Log => {
            ics_cli::commands::log::cmd_log()?;
        }
        Cmd::Diff { paths } => {
            ics_cli::commands::diff::cmd_diff(paths)?;
        }
        Cmd::Checkout { all, paths } => {
            ics_cli::commands::checkout::cmd_checkout(all, paths)?;
        }
        Cmd::Login { username, password } => {
            ics_cli::commands::login::cmd_login(username, password).await?;
        }
        Cmd::Push { paths } => {
            ics_cli::commands::push::cmd_push(paths).await?;
        }
        Cmd::Proposal { sub } => match sub {
            ProposalCmd::Submit {
                team_id,
                note_ids,
                rationale,
            } => {
                ics_cli::commands::proposal::cmd_proposal_submit(team_id, note_ids, rationale)
                    .await?;
            }
        },
        Cmd::Pull {
            all_tracked,
            paths,
            policy,
            force,
        } => {
            ics_cli::commands::pull::cmd_pull(all_tracked, paths, policy, force).await?;
        }
        Cmd::Vault { sub } => match sub {
            VaultCmd::Pull { slug, yes } => {
                ics_cli::commands::vault::cmd_vault_pull(slug, yes).await?;
            }
        },
    }
    Ok(())
}
