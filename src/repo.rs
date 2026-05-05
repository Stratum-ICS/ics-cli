use std::env;
use std::path::PathBuf;

pub fn find_repo(mut cwd: PathBuf) -> Option<PathBuf> {
    loop {
        if cwd.join(".ics").is_dir() {
            return Some(cwd);
        }
        if !cwd.pop() {
            return None;
        }
    }
}

pub fn find_repo_from_env() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    find_repo(cwd)
}

pub fn resolve_repo_root(path: Option<PathBuf>) -> std::io::Result<PathBuf> {
    match path {
        Some(p) => std::fs::canonicalize(p),
        None => env::current_dir(),
    }
}
