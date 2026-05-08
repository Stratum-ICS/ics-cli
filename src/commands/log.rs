use anyhow::Result;
use anyhow::Context;
use crate::commands::repo_root;
use crate::paths;
use crate::store::Store;

pub fn cmd_log() -> Result<()> {
    let root = repo_root()?;
    let store = Store::open(&paths::ics_dir(&root))?;
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
