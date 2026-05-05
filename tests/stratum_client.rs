use ics_cli::stratum::routes;
use ics_cli::stratum::StratumClient;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn post_proposal_ok() {
    let srv = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(routes::POST_PROPOSALS))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "p1",
            "status": "pending"
        })))
        .mount(&srv)
        .await;

    let client = StratumClient::new(srv.uri(), Some("token".into()));
    let out = client
        .post_proposal(&json!({
            "team_id": 1,
            "note_ids": [2],
            "rationale": "x".repeat(50)
        }))
        .await
        .unwrap();
    assert_eq!(out["id"], "p1");
}
