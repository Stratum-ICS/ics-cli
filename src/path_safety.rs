//! Enforce repository-relative paths for tree keys and CLI targets.

use crate::error::{IcsError, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Reject store-derived keys that could escape the worktree (`..`, absolute, odd segments).
pub fn assert_safe_tree_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(IcsError::Msg("empty tree path".into()));
    }
    if key.starts_with('/') || key.starts_with('\\') {
        return Err(IcsError::Msg(format!("invalid tree path: {key}")));
    }
    if key.contains('\\') {
        return Err(IcsError::Msg(format!("invalid tree path: {key}")));
    }
    for seg in key.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(IcsError::Msg(format!("invalid tree path: {key}")));
        }
    }
    Ok(())
}

/// Resolve a user-supplied path to an absolute path that stays under `repo_root`.
pub fn resolve_under_repo(repo_root: &Path, user: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(repo_root).map_err(|e| {
        IcsError::Msg(format!("cannot canonicalize repository root: {e}"))
    })?;

    let absolute = if user.is_absolute() {
        let u = fs::canonicalize(user).map_err(|e| {
            IcsError::Msg(format!("cannot resolve path {}: {e}", user.display()))
        })?;
        if !u.starts_with(&root) {
            return Err(IcsError::Msg(format!(
                "path {} is outside the repository",
                user.display()
            )));
        }
        u
    } else {
        let mut base = root.clone();
        for c in user.components() {
            match c {
                Component::CurDir => {}
                Component::ParentDir => {
                    if !base.pop() {
                        return Err(IcsError::Msg("path escapes repository".into()));
                    }
                    if !base.starts_with(&root) {
                        return Err(IcsError::Msg("path escapes repository".into()));
                    }
                }
                Component::Normal(x) => base.push(x),
                Component::RootDir | Component::Prefix(_) => {
                    return Err(IcsError::Msg("invalid path component".into()));
                }
            }
        }
        if !base.starts_with(&root) {
            return Err(IcsError::Msg("path outside repository".into()));
        }
        // Exists or not: canonicalize parent chain where possible
        match fs::canonicalize(&base) {
            Ok(c) => {
                if !c.starts_with(&root) {
                    return Err(IcsError::Msg("path outside repository".into()));
                }
                c
            }
            Err(_) => {
                let parent = base.parent().ok_or_else(|| {
                    IcsError::Msg("invalid path (no parent)".into())
                })?;
                let name = base.file_name().ok_or_else(|| {
                    IcsError::Msg("invalid path (no file name)".into())
                })?;
                let pcanon = fs::canonicalize(parent).map_err(|e| {
                    IcsError::Msg(format!("cannot resolve parent {}: {e}", parent.display()))
                })?;
                if !pcanon.starts_with(&root) {
                    return Err(IcsError::Msg("path outside repository".into()));
                }
                pcanon.join(name)
            }
        }
    };

    Ok(absolute)
}

/// POSIX tree key relative to `repo_root` for a user path.
pub fn user_path_to_tree_key(repo_root: &Path, user: &Path) -> Result<String> {
    let root = fs::canonicalize(repo_root).map_err(|e| {
        IcsError::Msg(format!("cannot canonicalize repository root: {e}"))
    })?;
    let resolved = resolve_under_repo(repo_root, user)?;
    let rel = resolved.strip_prefix(&root).map_err(|_| {
        IcsError::Msg("resolved path not under repository root".into())
    })?;
    let key = crate::worktree::posix_display(rel);
    assert_safe_tree_key(&key)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_dotdot_tree_key() {
        assert!(assert_safe_tree_key("foo/../bar").is_err());
    }

    #[test]
    fn user_cannot_escape_via_dotdot() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join(".ics")).unwrap();
        fs::write(root.join("a.md"), "").unwrap();
        assert!(user_path_to_tree_key(root, Path::new("../a.md")).is_err());
    }
}
