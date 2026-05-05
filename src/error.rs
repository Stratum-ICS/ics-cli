use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcsError {
    #[error("not a repository (no .ics directory found)")]
    NotRepository,
    #[error("no commits yet")]
    NoCommits,
    #[error("nothing to commit (clean working tree)")]
    NothingToCommit,
    #[error("already initialized")]
    AlreadyInitialized,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
    #[error("{0}")]
    Msg(String),
}

pub type Result<T> = std::result::Result<T, IcsError>;
