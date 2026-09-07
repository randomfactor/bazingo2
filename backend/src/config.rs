use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_url: String,
    pub session_secret: String,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self, String> {
        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| "demo-google-client-id".to_string());
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .unwrap_or_else(|_| "demo-google-client-secret".to_string());
        let google_redirect_url = std::env::var("GOOGLE_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:8000/auth/google/callback".to_string());
        let session_secret = std::env::var("SESSION_SECRET")
            .unwrap_or_else(|_| "development-session-secret".to_string());

        Ok(Self {
            google_client_id,
            google_client_secret,
            google_redirect_url,
            session_secret,
        })
    }
}
