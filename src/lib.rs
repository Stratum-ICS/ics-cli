//! Library crate for `ics` — local hybrid `.ics/` store.

pub mod commands;
pub mod paths;
pub mod objects;
pub mod store;
pub mod repo;
pub mod worktree;
pub mod commit;
pub mod error;
pub mod path_safety;
pub mod config;
pub mod auth_store;
pub mod frontmatter;
pub mod identity;
pub mod stratum;
pub mod sync;
