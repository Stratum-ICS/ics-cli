use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn iter_tracked_md(repo_root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk(repo_root, repo_root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(repo_root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name == ".ics" {
            continue;
        }
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(repo_root, &path, out)?;
        } else if meta.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("md")
        {
            let rel = path.strip_prefix(repo_root).unwrap_or(&path);
            out.push(rel.to_path_buf());
        }
    }
    Ok(())
}

pub fn posix_display(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn skips_ics_and_collects_md() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".ics")).unwrap();
        fs::create_dir_all(dir.path().join("notes")).unwrap();
        fs::write(dir.path().join("a.md"), "").unwrap();
        fs::write(dir.path().join("notes/b.md"), "").unwrap();
        fs::write(dir.path().join("skip.txt"), "").unwrap();

        let paths = iter_tracked_md(dir.path()).unwrap();
        let keys: Vec<String> = paths.iter().map(|p| posix_display(p)).collect();
        assert!(keys.contains(&"a.md".into()));
        assert!(keys.contains(&"notes/b.md".into()));
        assert_eq!(keys.len(), 2);
    }
}
