use anyhow::{Context, Result};
use crate::auth_store;
use crate::config::AppConfig;
use crate::stratum::StratumClient;
use serde_json::json;

pub async fn cmd_proposal_submit(
    team_id: u64,
    note_ids: Vec<u64>,
    rationale: String,
) -> Result<()> {
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
