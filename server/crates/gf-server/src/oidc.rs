use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::Extension;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreProviderMetadata,
};
use openidconnect::{
    AccessTokenHash, AdditionalClaims, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, UserInfoClaims,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

type DiscoveredOidcClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
    pub role_claim: String,
    pub role_map: Vec<(String, String)>,
    pub default_role: String,
    pub require_verified_email: bool,
    pub success_redirect: String,
    pub button_label: String,
    pub tenant_slug: String,
}

impl OidcConfig {
    pub fn from_env() -> Self {
        let role_map = std::env::var("GF_OIDC_ROLE_MAP")
            .unwrap_or_default()
            .split(',')
            .filter_map(|entry| {
                let (claim, role) = entry.split_once('=')?;
                let claim = claim.trim().to_string();
                let role = role.trim().to_string();
                if claim.is_empty() || !valid_tenant_role(&role) {
                    return None;
                }
                Some((claim, role))
            })
            .collect();
        Self {
            issuer: std::env::var("GF_OIDC_ISSUER")
                .ok()
                .filter(|v| !v.is_empty()),
            client_id: std::env::var("GF_OIDC_CLIENT_ID")
                .ok()
                .filter(|v| !v.is_empty()),
            client_secret: std::env::var("GF_OIDC_CLIENT_SECRET")
                .ok()
                .filter(|v| !v.is_empty()),
            redirect_uri: std::env::var("GF_OIDC_REDIRECT_URI")
                .ok()
                .filter(|v| !v.is_empty()),
            scopes: std::env::var("GF_OIDC_SCOPES")
                .unwrap_or_else(|_| "openid profile email".into())
                .split(|ch: char| ch.is_whitespace() || ch == ',')
                .filter(|scope| !scope.is_empty() && *scope != "openid")
                .map(str::to_string)
                .collect(),
            role_claim: std::env::var("GF_OIDC_ROLE_CLAIM").unwrap_or_else(|_| "groups".into()),
            role_map,
            default_role: std::env::var("GF_OIDC_DEFAULT_ROLE")
                .ok()
                .filter(|role| valid_tenant_role(role))
                .unwrap_or_else(|| "viewer".into()),
            require_verified_email: std::env::var("GF_OIDC_REQUIRE_VERIFIED_EMAIL")
                .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(true),
            success_redirect: std::env::var("GF_OIDC_SUCCESS_REDIRECT")
                .ok()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "/".into()),
            button_label: std::env::var("GF_OIDC_BUTTON_LABEL")
                .ok()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "Continue with SSO".into()),
            tenant_slug: std::env::var("GF_OIDC_TENANT_SLUG")
                .ok()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "default".into()),
        }
    }

    pub fn enabled(&self) -> bool {
        self.issuer.is_some() && self.client_id.is_some() && self.redirect_uri.is_some()
    }
}

fn valid_tenant_role(role: &str) -> bool {
    matches!(role, "viewer" | "member" | "admin" | "owner")
}

#[derive(Debug, Serialize)]
pub struct OidcPublicConfigResponse {
    enabled: bool,
    button_label: String,
}

pub async fn oidc_public_config(
    Extension(config): Extension<Arc<OidcConfig>>,
) -> axum::Json<OidcPublicConfigResponse> {
    axum::Json(OidcPublicConfigResponse {
        enabled: config.enabled(),
        button_label: config.button_label.clone(),
    })
}

fn oidc_error() -> (StatusCode, &'static str) {
    (StatusCode::INTERNAL_SERVER_ERROR, "oidc provider error")
}

async fn oidc_client(
    config: &OidcConfig,
) -> Result<(DiscoveredOidcClient, openidconnect::reqwest::Client), (StatusCode, &'static str)> {
    if !config.enabled() {
        return Err((StatusCode::NOT_FOUND, "oidc is not configured"));
    }
    let http_client = openidconnect::reqwest::ClientBuilder::new()
        .redirect(openidconnect::reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| oidc_error())?;
    let issuer = IssuerUrl::new(config.issuer.clone().expect("enabled checked"))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "invalid oidc issuer"))?;
    let provider_metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, "oidc discovery failed"))?;
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(config.client_id.clone().expect("enabled checked")),
        config.client_secret.clone().map(ClientSecret::new),
    )
    .set_redirect_uri(
        RedirectUrl::new(config.redirect_uri.clone().expect("enabled checked")).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid oidc redirect uri",
            )
        })?,
    );
    Ok((client, http_client))
}

#[derive(Debug, Deserialize)]
pub struct OidcStartParams {
    tenant_slug: Option<String>,
}

pub async fn oidc_start(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<OidcConfig>>,
    Query(params): Query<OidcStartParams>,
) -> Result<Redirect, (StatusCode, &'static str)> {
    let tenant_slug = params
        .tenant_slug
        .unwrap_or_else(|| config.tenant_slug.clone());
    if tenant_slug != config.tenant_slug {
        return Err((StatusCode::NOT_FOUND, "tenant not found"));
    }
    let tenant_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM tenants WHERE slug=$1)")
            .bind(&tenant_slug)
            .fetch_one(&pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "tenant lookup failed"))?;
    if !tenant_exists {
        return Err((StatusCode::NOT_FOUND, "tenant not found"));
    }

    let (client, _) = oidc_client(&config).await?;
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut authorization = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(pkce_challenge);
    for scope in &config.scopes {
        authorization = authorization.add_scope(Scope::new(scope.clone()));
    }
    let (auth_url, csrf_state, nonce) = authorization.url();
    gf_storage::create_oidc_auth_state(
        &pool,
        &tenant_slug,
        csrf_state.secret(),
        nonce.secret(),
        pkce_verifier.secret(),
        10,
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "oidc state persistence failed",
        )
    })?;
    Ok(Redirect::temporary(auth_url.as_str()))
}

#[derive(Debug, Deserialize)]
pub struct OidcCallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AllOtherClaims(HashMap<String, serde_json::Value>);
impl AdditionalClaims for AllOtherClaims {}

fn claim_values_at_path(claims: &HashMap<String, serde_json::Value>, path: &str) -> Vec<String> {
    fn collect_strings(value: &serde_json::Value, output: &mut Vec<String>) {
        match value {
            serde_json::Value::String(value) => output.push(value.clone()),
            serde_json::Value::Array(values) => values
                .iter()
                .for_each(|value| collect_strings(value, output)),
            _ => {}
        }
    }

    let mut current = None;
    for (index, segment) in path.split('.').enumerate() {
        current = if index == 0 {
            claims.get(segment)
        } else {
            current.and_then(|value: &serde_json::Value| value.get(segment))
        };
    }
    let mut output = Vec::new();
    if let Some(value) = current {
        collect_strings(value, &mut output);
    }
    output
}

fn payload_claims_after_verification(token: &str) -> HashMap<String, serde_json::Value> {
    let Some(payload) = token.split('.').nth(1) else {
        return HashMap::new();
    };
    let Ok(decoded) = URL_SAFE_NO_PAD.decode(payload) else {
        return HashMap::new();
    };
    serde_json::from_slice(&decoded).unwrap_or_default()
}

fn merged_oidc_claims(
    id_token_payload: HashMap<String, serde_json::Value>,
    userinfo: Option<&AllOtherClaims>,
) -> AllOtherClaims {
    let mut merged = id_token_payload;
    if let Some(userinfo) = userinfo {
        for (key, value) in &userinfo.0 {
            merged.insert(key.clone(), value.clone());
        }
    }
    AllOtherClaims(merged)
}

fn mapped_tenant_role(config: &OidcConfig, claims: Option<&AllOtherClaims>) -> Option<String> {
    let claims = claims?;
    let values = claim_values_at_path(&claims.0, &config.role_claim);
    config
        .role_map
        .iter()
        .find(|(claim, _)| values.iter().any(|value| value == claim))
        .map(|(_, role)| role.clone())
}

fn localized_or_username(
    preferred_username: Option<&openidconnect::EndUserUsername>,
    fallback: &str,
) -> String {
    preferred_username
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn fragment_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub async fn oidc_callback(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<OidcConfig>>,
    Query(params): Query<OidcCallbackParams>,
) -> Result<Redirect, (StatusCode, &'static str)> {
    if params.error.is_some() {
        return Err((StatusCode::UNAUTHORIZED, "oidc authorization failed"));
    }
    let code = params
        .code
        .ok_or((StatusCode::BAD_REQUEST, "missing authorization code"))?;
    let state = params
        .state
        .ok_or((StatusCode::BAD_REQUEST, "missing oidc state"))?;
    let stored = gf_storage::consume_oidc_auth_state(&pool, &state)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "oidc state lookup failed",
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid or expired oidc state"))?;

    let (client, http_client) = oidc_client(&config).await?;
    let pkce_verifier = PkceCodeVerifier::new(stored.pkce_verifier.clone());
    let token_response = client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|_| (StatusCode::BAD_GATEWAY, "oidc token endpoint unavailable"))?
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, "oidc token exchange failed"))?;
    let id_token = token_response.id_token().ok_or((
        StatusCode::BAD_GATEWAY,
        "oidc provider did not return an id token",
    ))?;
    let verifier = client.id_token_verifier();
    let nonce = Nonce::new(stored.nonce.clone());
    let claims = id_token
        .claims(&verifier, &nonce)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "oidc id token validation failed"))?;

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token.signing_alg().map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "unsupported oidc signing algorithm",
                )
            })?,
            id_token
                .signing_key(&verifier)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "oidc signing key unavailable"))?,
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "oidc access token hash validation failed",
            )
        })?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err((StatusCode::UNAUTHORIZED, "oidc access token hash mismatch"));
        }
    }

    let userinfo: Option<UserInfoClaims<AllOtherClaims, CoreGenderClaim>> = match client.user_info(
        token_response.access_token().to_owned(),
        Some(claims.subject().clone()),
    ) {
        Ok(request) => Some(
            request
                .request_async(&http_client)
                .await
                .map_err(|_| (StatusCode::BAD_GATEWAY, "oidc userinfo request failed"))?,
        ),
        Err(_) => None,
    };

    let email = userinfo
        .as_ref()
        .and_then(|value| value.email())
        .or_else(|| claims.email())
        .map(|value| value.as_str().to_string())
        .ok_or((StatusCode::FORBIDDEN, "oidc email claim is required"))?;
    let email_verified = userinfo
        .as_ref()
        .and_then(|value| value.email_verified())
        .or_else(|| claims.email_verified());
    if config.require_verified_email && email_verified != Some(true) {
        return Err((StatusCode::FORBIDDEN, "oidc email is not verified"));
    }

    let preferred_username = userinfo
        .as_ref()
        .and_then(|value| value.preferred_username())
        .or_else(|| claims.preferred_username());
    let display_name = localized_or_username(preferred_username, &email);
    let token_claims = payload_claims_after_verification(&id_token.to_string());
    let merged_claims = merged_oidc_claims(
        token_claims,
        userinfo.as_ref().map(|value| value.additional_claims()),
    );
    let mapped_role = mapped_tenant_role(&config, Some(&merged_claims));
    let user = gf_storage::upsert_oidc_user_identity(
        &pool,
        gf_storage::OidcUserIdentity {
            tenant_slug: &stored.tenant_slug,
            issuer: config.issuer.as_deref().expect("enabled checked"),
            subject: claims.subject().as_str(),
            email: &email,
            display_name: &display_name,
            mapped_role: mapped_role.as_deref(),
            default_role: &config.default_role,
        },
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "oidc user provisioning failed",
        )
    })?;
    if user.status != "active" {
        return Err((StatusCode::FORBIDDEN, "user is not active"));
    }
    let session = gf_storage::create_session(&pool, &user, 12)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "session creation failed"))?;

    let base = config.success_redirect.split('#').next().unwrap_or("/");
    let base = if base.is_empty() { "/" } else { base };
    let fragment = format!(
        "oidc_token={}&expires_at={}&user_id={}&tenant_id={}",
        fragment_encode(&session.token),
        fragment_encode(&session.expires_at.to_rfc3339()),
        session.user_id,
        session.tenant_id,
    );
    Ok(Redirect::to(&format!("{base}#{fragment}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_role_claims_are_extracted() {
        let claims = HashMap::from([(
            "realm_access".to_string(),
            serde_json::json!({"roles":["gf-admins","employees"]}),
        )]);
        assert_eq!(
            claim_values_at_path(&claims, "realm_access.roles"),
            vec!["gf-admins".to_string(), "employees".to_string()]
        );
    }

    #[test]
    fn role_mapping_uses_configured_group() {
        let mut config = OidcConfig::from_env();
        config.role_claim = "groups".into();
        config.role_map = vec![("gf-admins".into(), "admin".into())];
        let claims = AllOtherClaims(HashMap::from([(
            "groups".to_string(),
            serde_json::json!(["gf-admins"]),
        )]));
        assert_eq!(
            mapped_tenant_role(&config, Some(&claims)),
            Some("admin".into())
        );
    }
}
