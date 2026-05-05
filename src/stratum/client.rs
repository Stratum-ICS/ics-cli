use crate::stratum::errors::{http_error, StratumError};
use crate::stratum::routes;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, StatusCode};
use serde_json::Value;

#[derive(Clone)]
pub struct StratumClient {
    http: Client,
    base: String,
    token: Option<String>,
}

impl StratumClient {
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        Self {
            http: Client::new(),
            base: base_url.into(),
            token,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> Result<Value, StratumError> {
        let mut req = self
            .http
            .post(self.url(path))
            .header(CONTENT_TYPE, "application/json")
            .json(body);
        if let Some(t) = &self.token {
            req = req.header(AUTHORIZATION, format!("Bearer {t}"));
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if status == StatusCode::OK || status == StatusCode::CREATED {
            if text.is_empty() {
                return Ok(Value::Null);
            }
            return Ok(serde_json::from_str(&text)?);
        }
        Err(http_error(status, &text))
    }

    pub async fn get_json(&self, path: &str) -> Result<Value, StratumError> {
        let mut req = self.http.get(self.url(path));
        if let Some(t) = &self.token {
            req = req.header(AUTHORIZATION, format!("Bearer {t}"));
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if status == StatusCode::OK {
            return Ok(serde_json::from_str(&text)?);
        }
        Err(http_error(status, &text))
    }

    pub async fn put_json(&self, path: &str, body: &Value) -> Result<Value, StratumError> {
        let mut req = self
            .http
            .put(self.url(path))
            .header(CONTENT_TYPE, "application/json")
            .json(body);
        if let Some(t) = &self.token {
            req = req.header(AUTHORIZATION, format!("Bearer {t}"));
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if status.is_success() {
            if text.is_empty() {
                return Ok(Value::Null);
            }
            return Ok(serde_json::from_str(&text)?);
        }
        Err(http_error(status, &text))
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Value, StratumError> {
        let body = serde_json::json!({
            "username": username,
            "password": password,
        });
        self.post_json(routes::POST_AUTH_LOGIN, &body).await
    }

    pub async fn create_note(&self, body: &Value) -> Result<Value, StratumError> {
        self.post_json(routes::POST_NOTES, body).await
    }

    pub async fn update_note(&self, note_id: u64, body: &Value) -> Result<Value, StratumError> {
        self.put_json(&routes::put_note_path(note_id), body).await
    }

    pub async fn get_note(&self, note_id: u64) -> Result<Value, StratumError> {
        self.get_json(&routes::get_note_path(note_id)).await
    }

    pub async fn post_proposal(&self, body: &Value) -> Result<Value, StratumError> {
        self.post_json(routes::POST_PROPOSALS, body).await
    }

    pub async fn vault_pull(&self, slug: &str, body: &Value) -> Result<Value, StratumError> {
        let path = routes::post_vault_pull_path(slug);
        self.post_json(&path, body).await
    }
}
