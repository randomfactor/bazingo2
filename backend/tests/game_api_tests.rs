#[path = "../src/db/mod.rs"]
mod db;
mod game {
    #[path = "../../src/game/engine.rs"]
    pub mod engine;
    #[path = "../../src/game/services.rs"]
    pub mod services;
}

use crate::db::KVStore;
use db::DbPool;
use game::services::{
    create_game, current_turn, get_game, get_player, join_game, list_players, move_player,
    GameRecord, PlayerGameRecord,
};
use serde_json::json;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

#[tokio::test]
async fn game_lifecycle_persists_games_and_players() {
    let store = DbPool::new_in_memory("game_api_tests", "game_api_tests")
        .await
        .expect("in-memory database should initialize");

    let game = create_game(&store).await.expect("game should be created");
    assert_eq!(game.number_of_turns, 31);
    assert!(get_game(&store, &game.id).await.is_ok());

    let player = join_game(&store, &game.id, "user-1", "Player One")
        .await
        .expect("player should join");
    assert_eq!(player.user_id, "user-1");
    assert_eq!(player.last_turn_played, 0);

    let players = list_players(&store, &game.id).await.expect("players should list");
    assert_eq!(players.len(), 1);
    assert_eq!(players[0].name, "Player One");

    let loaded = get_player(&store, &game.id, "user-1")
        .await
        .expect("player should load");
    assert_eq!(loaded.gameboard, player.gameboard);
}

#[tokio::test]
async fn joining_the_same_game_twice_is_idempotent() {
    let store = DbPool::new_in_memory("game_api_tests_repeat", "game_api_tests_repeat")
        .await
        .expect("in-memory database should initialize");
    let game = create_game(&store).await.expect("game should be created");

    let first = join_game(&store, &game.id, "user-1", "Player One")
        .await
        .expect("first join should succeed");
    let second = join_game(&store, &game.id, "user-1", "Player One")
        .await
        .expect("second join should be idempotent");

    assert_eq!(first, second);
    assert_eq!(list_players(&store, &game.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn current_turn_is_negative_before_game_start() {
    let now = OffsetDateTime::now_utc();
    let game = GameRecord {
        id: "game-before-start".to_string(),
        name: "Before start".to_string(),
        starting_time: (now + Duration::minutes(5)).format(&Rfc3339).expect("timestamp should format"),
        ending_time: (now + Duration::minutes(60)).format(&Rfc3339).expect("timestamp should format"),
        number_of_turns: 31,
        turn_duration: 60,
        interim: 10,
        current_turn: 0,
        pieces: vec![],
        completed: false,
    };

    assert_eq!(current_turn(&game, now).expect("turn should compute"), -1);
}

#[tokio::test]
async fn move_requests_use_the_api_turn_index_and_are_rejected_out_of_window() {
    let store = DbPool::new_in_memory("game_api_turn_index", "game_api_turn_index")
        .await
        .expect("in-memory database should initialize");

    let now = OffsetDateTime::now_utc();
    let start = now - Duration::seconds(30);
    let game = GameRecord {
        id: "game-turn-index".to_string(),
        name: "Turn index game".to_string(),
        starting_time: start.format(&Rfc3339).expect("timestamp should format"),
        ending_time: (start + Duration::minutes(30)).format(&Rfc3339).expect("timestamp should format"),
        number_of_turns: 31,
        turn_duration: 60,
        interim: 10,
        current_turn: 0,
        pieces: vec![vec![vec![true, false, false], vec![false, false, false], vec![false, false, false]]],
        completed: false,
    };
    store
        .set("game:game-turn-index", json!(game))
        .await
        .expect("game should persist");

    let player = PlayerGameRecord {
        game_id: game.id.clone(),
        user_id: "user-turn-index".to_string(),
        name: "Player One".to_string(),
        gameboard: vec![vec![false; 5]; 5],
        score: 0,
        last_turn_played: 0,
    };
    store
        .set("game_player:game-turn-index_user-turn-index", json!(player))
        .await
        .expect("player should persist");

    let legal = move_player(&store, &game.id, "user-turn-index", 0, 0, 0).await;
    assert!(legal.is_ok(), "turn-zero move should be accepted for the active turn window");

    let illegal = move_player(&store, &game.id, "user-turn-index", 1, 0, 0).await;
    assert!(illegal.is_err(), "turn-one move should not be accepted before the correct turn window");
}
