use async_trait::async_trait;
use openidconnect::core::{
    CoreClient, CoreProviderMetadata, CoreResponseType,
};
use openidconnect::reqwest;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, RedirectUrl, Scope, TokenResponse,
};
use serde_json::Value;

use super::provider::{AuthUser, OAuthProvider, ProviderAuthRequest};
use crate::config::AuthConfig;

pub type DiscoveredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Clone)]
pub struct GoogleOAuthProvider {
    pub client: DiscoveredClient,
    pub http_client: reqwest::Client,
    #[allow(dead_code)]
    pub config: AuthConfig,
}

impl GoogleOAuthProvider {
    pub async fn new(config: AuthConfig) -> Result<Self, String> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|err| format!("failed to build reqwest client: {err}"))?;

        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new("https://accounts.google.com".to_string())
                .map_err(|err| format!("invalid Google issuer URL: {err}"))?,
            &http_client,
        )
        .await
        .map_err(|err| format!("failed to discover Google OIDC metadata: {err}"))?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.google_client_id.clone()),
            Some(ClientSecret::new(config.google_client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.google_redirect_url.clone())
                .map_err(|err| format!("invalid redirect URL: {err}"))?,
        );

        Ok(Self {
            client,
            http_client,
            config,
        })
    }
}

#[async_trait]
impl OAuthProvider for GoogleOAuthProvider {
    async fn authorization_url(&self, state: &str, nonce: &str) -> Result<ProviderAuthRequest, String> {
        let csrf_token = CsrfToken::new(state.to_string());
        let nonce_token = Nonce::new(nonce.to_string());

        let (authorize_url, _, _) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                move || csrf_token.clone(),
                move || nonce_token.clone(),
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        Ok(ProviderAuthRequest {
            authorization_url: authorize_url.to_string(),
            csrf_token: state.to_string(),
            nonce: nonce.to_string(),
        })
    }

    async fn verify_code_and_get_user(
        &self,
        code: &str,
        state: &str,
        nonce: &str,
    ) -> Result<AuthUser, String> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .map_err(|err| format!("failed to create token exchange: {err}"))?
            .request_async(&self.http_client)
            .await
            .map_err(|err| format!("failed to exchange code for token: {err}"))?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| "token response missing id_token".to_string())?;

        let claims = id_token
            .claims(&self.client.id_token_verifier(), &Nonce::new(nonce.to_string()))
            .map_err(|err| format!("failed to verify ID token: {err}"))?;

        if state.is_empty() {
            return Err("missing OAuth state".to_string());
        }

        let subject = claims.subject().to_string();
        let email = claims
            .email()
            .map(|email| email.as_str().to_string())
            .unwrap_or_default();
        let name = claims
            .name()
            .and_then(|name| name.get(None))
            .map(|name| name.to_string())
            .unwrap_or_else(|| email.clone());

        Ok(AuthUser {
            id: subject,
            email,
            name,
        })
    }

    fn provider_name(&self) -> &'static str {
        "google"
    }

    fn exchange_user_payload(&self, payload: Value) -> Result<AuthUser, String> {
        let id = payload.get("sub").and_then(Value::as_str).unwrap_or_default().to_string();
        let email = payload.get("email").and_then(Value::as_str).unwrap_or_default().to_string();
        let name = payload
            .get("name")
            .and_then(Value::as_str)
            .map(|value| value.to_string())
            .unwrap_or_else(|| email.clone());

        if id.is_empty() {
            return Err("Google user payload missing subject".to_string());
        }

        Ok(AuthUser { id, email, name })
    }
}
