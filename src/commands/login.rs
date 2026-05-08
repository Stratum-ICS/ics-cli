use anyhow::{Context, Result};
use crate::auth_store::{self, Credentials};
use crate::config::AppConfig;
use crate::stratum::StratumClient;
use serde_json::Value;

fn extract_access_token(v: &Value) -> Option<String> {
    v.get("access_token")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            v.get("token")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
        })
}

pub async fn cmd_login(username: String, password: Option<String>) -> Result<()> {
    let cfg = AppConfig::from_env()?;
    let password = password
        .or_else(|| std::env::var("STRATUM_PASSWORD").ok())
        .context("password required (--password or STRATUM_PASSWORD)")?;
    let client = StratumClient::new(cfg.stratum_base_url.clone(), None);
    let v = client.login(&username, &password).await?;
    let token = extract_access_token(&v).context("login response missing access_token/token")?;
    let cred = Credentials {
        access_token: token,
        base_url: Some(cfg.stratum_base_url.clone()),
    };
    auth_store::save_credentials(&cfg.credentials_path(), &cred)?;
    println!("saved credentials to {}", cfg.credentials_path().display());
    Ok(())
}
