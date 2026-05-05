// openapi_sha256: 5b9d199bf2f05da87795c365ca4285dd94456cb8ba5fa130d8334dd29e9a44e2
// stratum_rev: synthetic fixture (tests/fixtures/stratum-openapi.json) — replace with upstream OpenAPI + rev when available

/// `POST /api/auth/login`
pub const POST_AUTH_LOGIN: &str = "/api/auth/login";
/// `POST /api/notes`
pub const POST_NOTES: &str = "/api/notes";
/// `GET /api/notes/{note_id}`
pub fn get_note_path(note_id: u64) -> String {
    format!("/api/notes/{note_id}")
}
/// `PUT /api/notes/{note_id}`
pub fn put_note_path(note_id: u64) -> String {
    format!("/api/notes/{note_id}")
}
/// `POST /api/proposals`
pub const POST_PROPOSALS: &str = "/api/proposals";
/// `POST /api/vaults/{slug}/pull`
pub fn post_vault_pull_path(slug: &str) -> String {
    format!("/api/vaults/{slug}/pull")
}

/// Reject path injection in `{slug}` segment for vault URLs.
pub fn assert_safe_vault_slug(slug: &str) -> Result<(), super::StratumError> {
    use super::StratumError;
    if slug.is_empty()
        || slug.contains('/')
        || slug.contains('\\')
        || slug.contains("..")
    {
        return Err(StratumError::Msg(
            "vault slug must be a single path segment (no slashes or '..')".into(),
        ));
    }
    Ok(())
}
