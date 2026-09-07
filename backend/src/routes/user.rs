use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Serialize;

use crate::guards::auth_guard::AuthenticatedUser;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[get("/me")]
pub async fn me(user: AuthenticatedUser) -> (Status, Json<UserResponse>) {
    (
        Status::Ok,
        Json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        }),
    )
}
