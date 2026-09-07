use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket::State;
use serde_json::json;
use time::Duration;
use uuid::Uuid;

use crate::auth::google::GoogleOAuthProvider;
use crate::auth::provider::OAuthProvider;
use crate::db::KVStore;
use crate::LocalDbPool;

#[derive(FromForm)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

fn session_cookie(session_id: &str) -> Cookie<'static> {
    Cookie::build(("session_id", session_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build()
}

#[get("/google/login")]
pub async fn google_login(provider: &State<GoogleOAuthProvider>, cookies: &CookieJar<'_>) -> Redirect {
    let csrf_token = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();

    let auth_request = match provider.authorization_url(&csrf_token, &nonce).await {
        Ok(req) => req,
        Err(err) => {
            eprintln!("Google auth URL generation failed: {err}");
            return Redirect::to("/#/login?error=auth_failed");
        }
    };

    cookies.add(
        Cookie::build(("auth_csrf", csrf_token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::minutes(10))
            .build(),
    );
    cookies.add(
        Cookie::build(("auth_nonce", nonce))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::minutes(10))
            .build(),
    );

    Redirect::to(auth_request.authorization_url)
}

#[get("/google/callback?<query..>")]
pub async fn google_callback(
    query: CallbackQuery,
    provider: &State<GoogleOAuthProvider>,
    store: &State<LocalDbPool>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    if query.error.is_some() {
        return Redirect::to("/#/login?error=auth_failed");
    }

    let Some(code) = query.code else {
        return Redirect::to("/#/login?error=auth_failed");
    };
    let Some(state) = query.state else {
        return Redirect::to("/#/login?error=auth_failed");
    };

    let expected_csrf = cookies.get("auth_csrf").map(|cookie| cookie.value().to_string()).unwrap_or_default();
    let expected_nonce = cookies.get("auth_nonce").map(|cookie| cookie.value().to_string()).unwrap_or_default();

    if expected_csrf.is_empty() || expected_nonce.is_empty() || expected_csrf != state {
        return Redirect::to("/#/login?error=auth_failed");
    }

    let user = match provider.verify_code_and_get_user(&code, &state, &expected_nonce).await {
        Ok(user) => user,
        Err(err) => {
            eprintln!("Google user verification failed: {err}");
            return Redirect::to("/#/login?error=auth_failed");
        }
    };

    let session_id = Uuid::new_v4().to_string();
    let user_key = format!("user:google_{}", user.id);
    let session_key = format!("session:{session_id}");

    if let Err(err) = store.set(&user_key, json!({
        "id": user.id.clone(),
        "email": user.email.clone(),
        "name": user.name.clone(),
    })).await {
        eprintln!("failed to store user record: {err}");
        return Redirect::to("/#/login?error=auth_failed");
    }

    if let Err(err) = store.set(&session_key, json!({
        "id": user.id.clone(),
        "email": user.email.clone(),
        "name": user.name.clone(),
    })).await {
        eprintln!("failed to store session record: {err}");
        return Redirect::to("/#/login?error=auth_failed");
    }

    cookies.remove(Cookie::build(("auth_csrf", "")).path("/").build());
    cookies.remove(Cookie::build(("auth_nonce", "")).path("/").build());
    cookies.add(session_cookie(&session_id));

    Redirect::to("/#/")
}

#[post("/logout")]
pub async fn logout(cookies: &CookieJar<'_>, store: &State<LocalDbPool>) -> Status {
    if let Some(session_cookie) = cookies.get("session_id") {
        let session_id = session_cookie.value().to_string();
        let _ = store.set(&format!("session:{session_id}"), json!({ "expired": true })).await;
    }

    cookies.remove(Cookie::build(("session_id", "")).path("/").build());
    Status::Ok
}
