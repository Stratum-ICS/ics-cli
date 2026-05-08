use anyhow::{Context, Result};
use crate::auth_store;
use crate::config::AppConfig;
use crate::stratum::routes;
use crate::stratum::vault_client::{vault_pull_apply, vault_pull_preview};
use crate::stratum::StratumClient;
use serde_json::json;
use std::io::{stdin, IsTerminal};

pub async fn cmd_vault_pull(slug: String, yes: bool) -> Result<()> {
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
