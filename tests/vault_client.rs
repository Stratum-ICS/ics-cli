use ics_cli::stratum::routes;
use ics_cli::stratum::vault_client::{vault_pull_apply, vault_pull_preview};
use ics_cli::stratum::StratumClient;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn vault_pull_sets_confirm_flags() {
    let srv = MockServer::start().await;
    let slug = "demo";
    let route = routes::post_vault_pull_path(slug);

    Mock::given(method("POST"))
        .and(path(route.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(2)
        .mount(&srv)
        .await;

    let client = StratumClient::new(srv.uri(), Some("t".into()));
    let prev = vault_pull_preview(&client, slug, json!({})).await.unwrap();
    assert_eq!(prev["ok"], true);
    let next = vault_pull_apply(&client, slug, json!({})).await.unwrap();
    assert_eq!(next["ok"], true);
}
