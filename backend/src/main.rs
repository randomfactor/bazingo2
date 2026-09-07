#[macro_use]
extern crate rocket;

mod auth;
mod config;
mod db;
mod guards;
mod routes;

use rocket::State;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::{Request, Response};
use serde::Serialize;
use serde_json::Value;

use auth::google::GoogleOAuthProvider;
use config::AuthConfig;
use db::{DbPool, KVStore};
use routes::auth::{google_callback, google_login, logout};
use routes::static_files::{file_server, spa_fallback_catcher};
use routes::user::me;

type LocalDbPool = DbPool<surrealdb::engine::local::Db>;

#[derive(Serialize)]
struct DataResponse {
    message: String,
    visits: i64,
}

#[derive(Serialize)]
struct VisitResponse {
    total: i64,
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS Headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            "http://localhost:5173",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[get("/data")]
async fn get_data(store: &State<LocalDbPool>) -> Json<DataResponse> {
    let visits = store
        .increment("counter:global_home_visits", 1)
        .await
        .unwrap_or(0);

    Json(DataResponse {
        message: "Hello from the Rocket backend!".to_string(),
        visits,
    })
}

#[get("/get-visits")]
async fn get_visits(store: &State<LocalDbPool>) -> Json<VisitResponse> {
    let total = match store.get("counter:global_home_visits").await {
        Ok(Some(Value::Number(number))) => number.as_i64().unwrap_or(0),
        Ok(Some(Value::String(value))) => value.parse::<i64>().unwrap_or(0),
        Ok(_) => 0,
        Err(_) => 0,
    };

    Json(VisitResponse { total })
}

#[get("/visit")]
async fn visit(store: &State<LocalDbPool>) -> Json<VisitResponse> {
    let total = store
        .increment("counter:global_home_visits", 1)
        .await
        .unwrap_or(0);

    Json(VisitResponse { total })
}

#[options("/data")]
fn options_data() -> rocket::http::Status {
    rocket::http::Status::NoContent
}

#[launch]
async fn rocket() -> _ {
    if cfg!(debug_assertions) { // load env vars from file, but not in production
        dotenvy::from_filename(".env.local").ok();
    }

    let store: LocalDbPool = DbPool::from_env()
        .await
        .expect("failed to initialize SurrealDB connection pool");
    let auth_config = AuthConfig::from_env().expect("failed to load auth config");
    let provider = GoogleOAuthProvider::new(auth_config.clone())
        .await
        .expect("failed to initialize Google OAuth provider");

    rocket::build()
        .manage(store)
        .manage(auth_config)
        .manage(provider)
        .attach(CORS)
        .mount("/api", routes![get_data, get_visits, visit, me, options_data])
        .mount("/auth", routes![google_login, google_callback, logout])
        .mount("/", file_server())
        .register("/", catchers![spa_fallback_catcher])
}
