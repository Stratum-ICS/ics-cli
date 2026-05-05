use crate::stratum::client::StratumClient;
use crate::stratum::errors::StratumError;
use serde_json::{json, Value};

pub async fn vault_pull_preview(
    client: &StratumClient,
    slug: &str,
    extra: Value,
) -> Result<Value, StratumError> {
    let mut body = if extra.is_null() {
        json!({})
    } else {
        extra
    };
    let obj = body
        .as_object_mut()
        .ok_or_else(|| StratumError::Msg("vault pull body must be a JSON object".into()))?;
    obj.insert("confirm".into(), json!(false));
    client.vault_pull(slug, &body).await
}

pub async fn vault_pull_apply(
    client: &StratumClient,
    slug: &str,
    mut body: Value,
) -> Result<Value, StratumError> {
    let obj = body
        .as_object_mut()
        .ok_or_else(|| StratumError::Msg("vault pull body must be a JSON object".into()))?;
    obj.insert("confirm".into(), json!(true));
    client.vault_pull(slug, &body).await
}
