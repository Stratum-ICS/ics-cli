pub mod checkout;
pub mod commit;
pub mod diff;
pub mod init;
pub mod log;
pub mod login;
pub mod proposal;
pub mod pull;
pub mod push;
pub mod status;
pub mod vault;

use crate::error::{IcsError, Result};
use std::path::PathBuf;

#[allow(clippy::useless_conversion)]
pub fn repo_root() -> Result<PathBuf> {
    crate::repo::find_repo_from_env()
        .ok_or_else(|| IcsError::NotRepository)
        .map_err(IcsError::from)
}
