#[path = "../src/game/mod.rs"]
mod game_engine;

use game_engine::{
    all_gamepieces, compute_turn_score, empty_gameboard, gamepiece_from_octal, place_piece,
    BzGame, BzPlayer, GameEngineError, FINAL_EMPTY_BOARD_BONUS,
};

#[test]
fn new_game_contains_all_31_pieces_once() {
    let game = BzGame::new(31);
    let mut actual = game.pieces;
    let mut expected = all_gamepieces();
    actual.sort_by_key(|piece| format!("{piece:?}"));
    expected.sort_by_key(|piece| format!("{piece:?}"));
    assert_eq!(actual, expected);
}

#[test]
fn placement_handles_normal_edge_and_conflicting_moves() {
    let game = BzGame {
        pieces: vec![gamepiece_from_octal([0, 2, 0])],
        turns: 2,
    };
    let mut player = BzPlayer::new();
    assert!(place_piece(&game, &player, 1, -1, 0).is_ok());
    player.gameboard[1][1] = true;
    assert_eq!(
        place_piece(&game, &player, 1, 0, 0),
        Err(GameEngineError::IllegalPlacement)
    );
}

#[test]
fn scoring_removes_a_three_by_three_block() {
    let game = BzGame {
        pieces: all_gamepieces(),
        turns: 2,
    };
    let player = BzPlayer::new();
    let board = vec![
        vec![true, true, true, false, false],
        vec![true, true, true, false, false],
        vec![true, true, true, false, false],
        vec![false; 5],
        vec![false; 5],
    ];
    let result = compute_turn_score(&game, &player, 1, board).unwrap();
    assert_eq!(result.score, 45);
    assert!(result.gameboard.iter().flatten().all(|&pip| !pip));
}

#[test]
fn missed_turns_and_final_empty_bonus_are_applied() {
    let game = BzGame {
        pieces: all_gamepieces(),
        turns: 3,
    };
    let result = compute_turn_score(&game, &BzPlayer::new(), 3, empty_gameboard()).unwrap();
    assert_eq!(result.score, -14 + FINAL_EMPTY_BOARD_BONUS);
}
