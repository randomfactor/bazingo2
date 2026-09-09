use crate::game::services::{
    self, current_turn, get_game, get_player, join_game, list_games, list_players, move_player,
    GamePlayerSummary, GameRecord, PlayerGameRecord, ServiceError,
};
use crate::guards::auth_guard::AuthenticatedUser;
use crate::LocalDbPool;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Deserialize, FromForm)]
pub struct GameListQuery {
    pub limit: Option<usize>,
    pub start_after: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveRequest {
    pub turn_number: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize)]
pub struct ActiveGameResponse {
    pub id: String,
    pub name: String,
    pub starting_time: String,
    pub current_turn: i32,
    pub player_count: usize,
}

#[derive(Debug, Serialize)]
pub struct PastGameResponse {
    pub id: String,
    pub name: String,
    pub starting_time: String,
    pub ending_time: String,
    pub number_of_turns: u32,
    pub player_count: usize,
    pub highest_score: i32,
}

#[derive(Debug, Serialize)]
pub struct CreatedGameResponse {
    pub id: String,
    pub name: String,
    pub starting_time: String,
}

#[derive(Debug, Serialize)]
pub struct GameInfoResponse {
    pub id: String,
    pub name: String,
    pub starting_time: String,
    pub ending_time: String,
    pub number_of_turns: u32,
    pub turn_duration: i64,
    pub interim: i64,
    pub current_turn: i32,
    pub pieces: Option<Vec<crate::game::BzGamepiece>>,
    pub completed: bool,
}

#[derive(Debug, Serialize)]
pub struct PlayerStateResponse {
    pub game_id: String,
    pub user_id: String,
    pub score: i32,
    pub last_turn_played: u32,
    pub gameboard: crate::game::BzGameboard,
}

#[derive(Debug, Serialize)]
pub struct MyGameResponse {
    pub id: String,
    pub name: String,
    pub number_of_turns: u32,
    pub current_turn: i32,
    pub current_piece: Vec<crate::game::BzGamepiece>,
    pub gameboard: crate::game::BzGameboard,
    pub player_turn: u32,
    pub time_remaining_seconds: i64,
}

#[get("/games/active?<query..>")]
pub async fn active_games(
    query: Option<GameListQuery>,
    store: &State<LocalDbPool>,
) -> Result<Json<Vec<ActiveGameResponse>>, Status> {
    let query = query.unwrap_or(GameListQuery { limit: None, start_after: None });
    let limit = query.limit.unwrap_or(20).min(100);
    if limit == 0 {
        return Ok(Json(Vec::new()));
    }
    let now = OffsetDateTime::now_utc();
    let mut response = Vec::new();
    for game in list_games(store.inner()).await.map_err(service_status)? {
        if game.completed || services::parse_time(&game.ending_time).map_err(service_status)? <= now {
            continue;
        }
        if query.start_after.as_ref().is_some_and(|value| game.starting_time <= *value) {
            continue;
        }
        let players = list_players(store.inner(), &game.id).await.map_err(service_status)?;
        let turn = current_turn(&game, now).map_err(service_status)?;
        response.push(ActiveGameResponse {
            id: game.id,
            name: game.name,
            starting_time: game.starting_time,
            current_turn: turn,
            player_count: players.len(),
        });
        if response.len() >= limit {
            break;
        }
    }
    Ok(Json(response))
}

#[get("/games/past?<query..>")]
pub async fn past_games(
    query: Option<GameListQuery>,
    store: &State<LocalDbPool>,
) -> Result<Json<Vec<PastGameResponse>>, Status> {
    let query = query.unwrap_or(GameListQuery { limit: None, start_after: None });
    let limit = query.limit.unwrap_or(20).min(100);
    if limit == 0 {
        return Ok(Json(Vec::new()));
    }
    let now = OffsetDateTime::now_utc();
    let mut response = Vec::new();
    for game in list_games(store.inner()).await.map_err(service_status)? {
        if !game.completed && services::parse_time(&game.ending_time).map_err(service_status)? > now {
            continue;
        }
        if query.start_after.as_ref().is_some_and(|value| game.starting_time <= *value) {
            continue;
        }
        let players = list_players(store.inner(), &game.id).await.map_err(service_status)?;
        let mut highest_score = 0;
        for player in players.iter() {
            if let Ok(state) = get_player(store.inner(), &game.id, &player.id).await {
                highest_score = highest_score.max(state.score);
            }
        }
        response.push(PastGameResponse {
            id: game.id,
            name: game.name,
            starting_time: game.starting_time,
            ending_time: game.ending_time,
            number_of_turns: game.number_of_turns,
            player_count: players.len(),
            highest_score,
        });
        if response.len() >= limit {
            break;
        }
    }
    Ok(Json(response))
}

#[post("/games")]
pub async fn create_game(
    _user: AuthenticatedUser,
    store: &State<LocalDbPool>,
) -> Result<Custom<Json<CreatedGameResponse>>, Status> {
    let game = services::create_game(store.inner()).await.map_err(service_status)?;
    Ok(Custom(
        Status::Created,
        Json(CreatedGameResponse {
            id: game.id,
            name: game.name,
            starting_time: game.starting_time,
        }),
    ))
}

#[get("/games/info/<id>")]
pub async fn game_info(
    _user: AuthenticatedUser,
    id: &str,
    store: &State<LocalDbPool>,
) -> Result<Json<GameInfoResponse>, Status> {
    let game = get_game(store.inner(), id).await.map_err(service_status)?;
    let turn_index = current_turn(&game, OffsetDateTime::now_utc()).map_err(service_status)?;
    let in_progress = turn_index <= game.number_of_turns as i32;
    Ok(Json(GameInfoResponse {
        id: game.id.clone(),
        name: game.name.clone(),
        starting_time: game.starting_time.clone(),
        ending_time: game.ending_time.clone(),
        number_of_turns: game.number_of_turns,
        turn_duration: game.turn_duration,
        interim: game.interim,
        current_turn: turn_index,
        pieces: if in_progress { None } else { Some(game.pieces.clone()) },
        completed: game.completed,
    }))
}

#[get("/games/me")]
pub async fn my_game(
    user: AuthenticatedUser,
    store: &State<LocalDbPool>,
) -> Result<Json<MyGameResponse>, Status> {
    let now = OffsetDateTime::now_utc();
    for game in list_games(store.inner()).await.map_err(service_status)? {
        if let Ok(player) = get_player(store.inner(), &game.id, &user.id).await {
            if !game.completed && services::parse_time(&game.ending_time).map_err(service_status)? > now {
                let turn_index = current_turn(&game, now).map_err(service_status)?;
                let current_piece = if turn_index < 0 {
                    Vec::new()
                } else {
                    game.pieces
                        .get(turn_index as usize)
                        .cloned()
                        .map(|piece| vec![piece])
                        .unwrap_or_default()
                };
                let board = if turn_index < 0 {
                    crate::game::empty_gameboard()
                } else {
                    player.gameboard.clone()
                };
                let slot = game.turn_duration + game.interim;
                let time_remaining = if turn_index < 0 {
                    (services::parse_time(&game.starting_time).map_err(service_status)? - now)
                        .whole_seconds()
                        .max(0)
                } else {
                    let next_window_start = services::parse_time(&game.starting_time).map_err(service_status)?
                        + Duration::seconds(((turn_index + 1) as i64) * slot);
                    (next_window_start - now).whole_seconds().max(0)
                };

                return Ok(Json(MyGameResponse {
                    id: game.id,
                    name: game.name,
                    number_of_turns: game.number_of_turns,
                    current_turn: turn_index,
                    current_piece,
                    gameboard: board,
                    player_turn: player.last_turn_played,
                    time_remaining_seconds: time_remaining,
                }));
            }
        }
    }
    Ok(Json(MyGameResponse {
        id: String::new(),
        name: String::new(),
        number_of_turns: 0,
        current_turn: -1,
        current_piece: Vec::new(),
        gameboard: crate::game::empty_gameboard(),
        player_turn: 0,
        time_remaining_seconds: 0,
    }))
}

#[post("/games/join/<id>")]
pub async fn join(
    user: AuthenticatedUser,
    id: &str,
    store: &State<LocalDbPool>,
) -> Result<Json<PlayerStateResponse>, Status> {
    let player = join_game(store.inner(), id, &user.id, &user.name).await.map_err(service_status)?;
    Ok(Json(player_response(player)))
}

#[get("/games/players/<id>")]
pub async fn players(
    _user: AuthenticatedUser,
    id: &str,
    store: &State<LocalDbPool>,
) -> Result<Json<Vec<GamePlayerSummary>>, Status> {
    get_game(store.inner(), id).await.map_err(service_status)?;
    Ok(Json(list_players(store.inner(), id).await.map_err(service_status)?))
}

#[post("/games/move/<id>", data = "<request>")]
pub async fn make_move(
    user: AuthenticatedUser,
    id: &str,
    request: Json<MoveRequest>,
    store: &State<LocalDbPool>,
) -> Result<Json<PlayerStateResponse>, Status> {
    let player = move_player(
        store.inner(),
        id,
        &user.id,
        request.turn_number,
        request.x,
        request.y,
    )
    .await
    .map_err(service_status)?;
    Ok(Json(player_response(player)))
}

fn player_response(player: PlayerGameRecord) -> PlayerStateResponse {
    PlayerStateResponse {
        game_id: player.game_id,
        user_id: player.user_id,
        score: player.score,
        last_turn_played: player.last_turn_played,
        gameboard: player.gameboard,
    }
}

fn service_status(error: ServiceError) -> Status {
    match error {
        ServiceError::NotFound => Status::NotFound,
        ServiceError::Conflict(_) => Status::Conflict,
        ServiceError::Invalid(_) => Status::BadRequest,
        ServiceError::Storage(_) => Status::InternalServerError,
    }
}

#[allow(dead_code)]
fn _record_types_are_serializable(_: GameRecord, _: PlayerGameRecord) {}
