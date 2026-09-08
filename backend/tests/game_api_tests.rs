#[path = "../src/db/mod.rs"]
mod db;
mod game {
    #[path = "../../src/game/engine.rs"]
    pub mod engine;
    #[path = "../../src/game/services.rs"]
    pub mod services;
}

use db::DbPool;
use game::services::{create_game, get_game, get_player, join_game, list_players};

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
