use super::engine::{place_piece, BzGame, BzGameboard, BzGamepiece, BzPlayer, GameEngineError};
use crate::db::KVStore;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

pub const DEFAULT_NUMBER_OF_TURNS: u32 = 31;
pub const DEFAULT_TURN_DURATION_SECONDS: i64 = 60;
pub const DEFAULT_INTERIM_SECONDS: i64 = 10;
pub const MAX_PLAYERS: usize = 100;

#[derive(Debug)]
pub enum ServiceError {
    NotFound,
    Conflict(String),
    Invalid(String),
    Storage(String),
}

impl From<GameEngineError> for ServiceError {
    fn from(error: GameEngineError) -> Self {
        Self::Invalid(error.to_string())
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(error: serde_json::Error) -> Self {
        Self::Storage(error.to_string())
    }
}

impl From<time::error::Format> for ServiceError {
    fn from(error: time::error::Format) -> Self {
        Self::Storage(error.to_string())
    }
}

impl From<time::error::InvalidFormatDescription> for ServiceError {
    fn from(error: time::error::InvalidFormatDescription) -> Self {
        Self::Storage(error.to_string())
    }
}

impl From<time::error::ComponentRange> for ServiceError {
    fn from(error: time::error::ComponentRange) -> Self {
        Self::Storage(error.to_string())
    }
}

impl From<time::error::Parse> for ServiceError {
    fn from(error: time::error::Parse) -> Self {
        Self::Storage(error.to_string())
    }
}

impl From<crate::db::Error> for ServiceError {
    fn from(error: crate::db::Error) -> Self {
        Self::Storage(error.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub id: String,
    pub name: String,
    pub starting_time: String,
    pub ending_time: String,
    pub number_of_turns: u32,
    pub turn_duration: i64,
    pub interim: i64,
    pub current_turn: u32,
    pub pieces: Vec<BzGamepiece>,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerGameRecord {
    pub game_id: String,
    pub user_id: String,
    pub name: String,
    pub gameboard: BzGameboard,
    pub score: i32,
    pub last_turn_played: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerSummary {
    pub id: String,
    pub name: String,
}

pub fn game_key(id: &str) -> String {
    format!("game:{id}")
}

pub fn player_key(game_id: &str, user_id: &str) -> String {
    format!("game_player:{game_id}_{user_id}")
}

pub async fn create_game<S: KVStore>(store: &S) -> Result<GameRecord, ServiceError> {
    let id = Uuid::new_v4().to_string();
    let starting = OffsetDateTime::now_utc() + Duration::minutes(2);
    let ending = starting + Duration::seconds(
        DEFAULT_NUMBER_OF_TURNS as i64
            * (DEFAULT_TURN_DURATION_SECONDS + DEFAULT_INTERIM_SECONDS),
    );
    let engine_game = BzGame::new(DEFAULT_NUMBER_OF_TURNS);
    let game = GameRecord {
        id: id.clone(),
        name: format!("Game {}", &id[..8]),
        starting_time: starting.format(&Rfc3339)?,
        ending_time: ending.format(&Rfc3339)?,
        number_of_turns: DEFAULT_NUMBER_OF_TURNS,
        turn_duration: DEFAULT_TURN_DURATION_SECONDS,
        interim: DEFAULT_INTERIM_SECONDS,
        current_turn: 0,
        pieces: engine_game.pieces,
        completed: false,
    };
    save_json(store, &game_key(&id), &game).await?;
    Ok(game)
}

pub async fn get_game<S: KVStore>(store: &S, id: &str) -> Result<GameRecord, ServiceError> {
    load_json(store, &game_key(id)).await?.ok_or(ServiceError::NotFound)
}

pub async fn list_games<S: KVStore>(store: &S) -> Result<Vec<GameRecord>, ServiceError> {
    let mut games = Vec::new();
    for value in store.list_table("game").await? {
        if let Ok(game) = serde_json::from_value::<GameRecord>(value) {
            games.push(game);
        }
    }
    games.sort_by(|left, right| left.starting_time.cmp(&right.starting_time));
    Ok(games)
}

pub async fn join_game<S: KVStore>(
    store: &S,
    game_id: &str,
    user_id: &str,
    user_name: &str,
) -> Result<PlayerGameRecord, ServiceError> {
    let game = get_game(store, game_id).await?;
    let now = OffsetDateTime::now_utc();
    if now >= parse_time(&game.ending_time)? || game.completed {
        return Err(ServiceError::Conflict("game is completed".to_string()));
    }

    let key = player_key(game_id, user_id);
    if let Some(existing) = load_json::<PlayerGameRecord, S>(store, &key).await? {
        return Ok(existing);
    }

    let players = list_players(store, game_id).await?;
    if players.len() >= MAX_PLAYERS {
        return Err(ServiceError::Conflict("game is full".to_string()));
    }

    let player = PlayerGameRecord {
        game_id: game_id.to_string(),
        user_id: user_id.to_string(),
        name: user_name.to_string(),
        gameboard: super::engine::empty_gameboard(),
        score: 0,
        last_turn_played: 0,
    };
    save_json(store, &key, &player).await?;
    Ok(player)
}

pub async fn list_players<S: KVStore>(
    store: &S,
    game_id: &str,
) -> Result<Vec<GamePlayerSummary>, ServiceError> {
    let mut players = Vec::new();
    for value in store.list_table("game_player").await? {
        if let Ok(player) = serde_json::from_value::<PlayerGameRecord>(value) {
            if player.game_id == game_id {
                players.push(GamePlayerSummary { id: player.user_id, name: player.name });
            }
        }
    }
    players.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(players)
}

pub async fn get_player<S: KVStore>(
    store: &S,
    game_id: &str,
    user_id: &str,
) -> Result<PlayerGameRecord, ServiceError> {
    load_json(store, &player_key(game_id, user_id)).await?.ok_or(ServiceError::NotFound)
}

pub fn current_turn(game: &GameRecord, now: OffsetDateTime) -> Result<i32, ServiceError> {
    let start = parse_time(&game.starting_time)?;
    let end = parse_time(&game.ending_time)?;
    if now < start {
        return Ok(-1);
    }
    if now >= end {
        return Ok(game.number_of_turns as i32);
    }
    let slot = game.turn_duration + game.interim;
    Ok(((now - start).whole_seconds() / slot).max(0) as i32)
}

pub async fn move_player<S: KVStore>(
    store: &S,
    game_id: &str,
    user_id: &str,
    turn_number: i32,
    x: i32,
    y: i32,
) -> Result<PlayerGameRecord, ServiceError> {
    let mut game = get_game(store, game_id).await?;
    let mut player = get_player(store, game_id, user_id).await?;

    if turn_number < 0 {
        return Err(ServiceError::Conflict("turn must be non-negative".to_string()));
    }

    let expected_turn = current_turn(&game, OffsetDateTime::now_utc())?;
    if expected_turn < 0 {
        return Err(ServiceError::Conflict("game has not started".to_string()));
    }
    if expected_turn >= game.number_of_turns as i32 {
        return Err(ServiceError::Conflict("game is completed".to_string()));
    }
    if turn_number != expected_turn {
        return Err(ServiceError::Conflict("move is not for the current turn".to_string()));
    }

    let engine_turn = turn_number + 1;
    let engine_game = BzGame { pieces: game.pieces.clone(), turns: game.number_of_turns };
    let engine_player = BzPlayer {
        gameboard: player.gameboard.clone(),
        last_turn_played: player.last_turn_played,
    };
    let board = place_piece(&engine_game, &engine_player, engine_turn as u32, x, y)?;
    let scored = super::engine::compute_turn_score(
        &engine_game,
        &engine_player,
        engine_turn as u32,
        board,
    )?;

    player.gameboard = scored.gameboard;
    player.score += scored.score;
    player.last_turn_played = turn_number as u32;
    game.current_turn = turn_number as u32;
    if game.current_turn == game.number_of_turns {
        game.completed = true;
    }
    save_json(store, &player_key(game_id, user_id), &player).await?;
    save_json(store, &game_key(game_id), &game).await?;
    Ok(player)
}

pub fn parse_time(value: &str) -> Result<OffsetDateTime, ServiceError> {
    Ok(OffsetDateTime::parse(value, &Rfc3339)?)
}

async fn load_json<T, S>(store: &S, key: &str) -> Result<Option<T>, ServiceError>
where
    T: for<'de> Deserialize<'de>,
    S: KVStore,
{
    let value = store.get(key).await?;
    value.map(serde_json::from_value).transpose().map_err(ServiceError::from)
}

async fn save_json<T, S>(store: &S, key: &str, value: &T) -> Result<(), ServiceError>
where
    T: Serialize,
    S: KVStore,
{
    let value = serde_json::to_value(value)?;
    store.set(key, value).await?;
    Ok(())
}

