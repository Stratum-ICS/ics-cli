use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StratumError {
    #[error("stratum HTTP {status}: {detail}")]
    Http { status: u16, detail: String },
    #[error("stratum request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("invalid response JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Msg(String),
}

pub fn http_error(status: StatusCode, body: &str) -> StratumError {
    let detail = format_body_hint(status, body);
    StratumError::Http {
        status: status.as_u16(),
        detail,
    }
}

fn format_body_hint(status: StatusCode, body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(body) {
        let mut s = v.to_string();
        if let Some(hints) = v.get("conflict_hints") {
            s.push_str("\nconflict_hints: ");
            s.push_str(&hints.to_string());
        }
        if let Some(detail) = v.get("detail") {
            s.push_str("\ndetail: ");
            s.push_str(&detail.to_string());
        }
        return format!("{status} — {s}");
    }
    format!("{status} — {body}")
}
