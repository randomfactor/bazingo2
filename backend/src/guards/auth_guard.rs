use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use serde::{Deserialize, Serialize};

use crate::db::KVStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let session_id = match request.cookies().get("session_id") {
            Some(cookie) => cookie.value().to_string(),
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        if session_id.is_empty() {
            return Outcome::Error((Status::Unauthorized, ()));
        }

        let store = match request.rocket().state::<crate::LocalDbPool>() {
            Some(store) => store,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let Some(user_json) = (match store.get(&format!("session:{session_id}")).await {
            Ok(value) => value,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        }) else {
            return Outcome::Error((Status::Unauthorized, ()));
        };

        if user_json.get("expired").and_then(serde_json::Value::as_bool).unwrap_or(false) {
            return Outcome::Error((Status::Unauthorized, ()));
        }

        let Ok(user) = serde_json::from_value::<AuthenticatedUser>(user_json) else {
            return Outcome::Error((Status::Unauthorized, ()));
        };

        Outcome::Success(user)
    }
}
