use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ProviderAuthRequest {
    pub authorization_url: String,
    #[allow(dead_code)]
    pub csrf_token: String,
    #[allow(dead_code)]
    pub nonce: String,
}

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    async fn authorization_url(&self, state: &str, nonce: &str) -> Result<ProviderAuthRequest, String>;
    async fn verify_code_and_get_user(
        &self,
        code: &str,
        state: &str,
        nonce: &str,
    ) -> Result<AuthUser, String>;
    #[allow(dead_code)]
    fn provider_name(&self) -> &'static str;
    #[allow(dead_code)]
    fn exchange_user_payload(&self, payload: Value) -> Result<AuthUser, String>;
}
