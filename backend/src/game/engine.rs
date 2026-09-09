#![allow(dead_code)]

use rand::seq::SliceRandom;
use std::fmt;

pub type BzGameboard = Vec<Vec<bool>>;
pub type BzGamepiece = Vec<Vec<bool>>;

pub const BOARD_SIZE: usize = 5;
pub const PIECE_SIZE: usize = 3;
pub const FAILURE_PENALTY: i32 = -7;
pub const FINAL_EMPTY_BOARD_BONUS: i32 = 15;

const PIECE_PATTERNS: [[u8; 3]; 35] = [
    [0, 2, 0], // single pip
    [2, 2, 0], [0, 3, 0], [0, 7, 0], [2, 2, 2], // 2-pip and 3-pip straight lines
    [1, 2, 0], [4, 2, 0], [1, 2, 4], [4, 2, 1], // 2-pip and 3-pip diagonals
    [2, 3, 0], [0, 3, 2], [0, 6, 2], [2, 6, 0], // corners
    [2, 6, 4], [4, 6, 2], [6, 3, 0], [3, 6, 0], // snakes
    [1, 7, 0], [0, 7, 1], [4, 7, 0], [0, 7, 4], // horizontal ells
    [3, 2, 2], [6, 2, 2], [2, 2, 3], [2, 2, 6], // vertical ells
    [2, 3, 2], [0, 7, 2], [2, 6, 2], [2, 7, 0], // pyramids
    [3, 2, 3], [0, 7, 5], [6, 2, 6], [5, 7, 0], // arches
    [5, 2, 5], [2, 7, 2], // checkerboard and cross
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEngineError {
    InvalidBoard,
    InvalidPiece,
    InvalidTurn,
    TurnAlreadyPlayed,
    TurnExceedsGame,
    IllegalPlacement,
}

impl fmt::Display for GameEngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for GameEngineError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BzGame {
    pub pieces: Vec<BzGamepiece>,
    pub turns: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BzPlayer {
    pub gameboard: BzGameboard,
    pub last_turn_played: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnScore {
    pub score: i32,
    pub gameboard: BzGameboard,
    pub scored_blocks: Vec<(usize, usize, usize, usize)>,
}

pub fn empty_gameboard() -> BzGameboard {
    vec![vec![false; BOARD_SIZE]; BOARD_SIZE]
}

pub fn gamepiece_from_octal(rows: [u8; PIECE_SIZE]) -> BzGamepiece {
    rows.into_iter()
        .map(|row| (0..PIECE_SIZE).map(|column| row & (1 << column) != 0).collect())
        .collect()
}

pub fn all_gamepieces() -> Vec<BzGamepiece> {
    PIECE_PATTERNS
        .into_iter()
        .take(31)
        .map(gamepiece_from_octal)
        .collect()
}

impl BzGame {
    pub fn new(turns: u32) -> Self {
        let mut pieces = all_gamepieces();
        pieces.shuffle(&mut rand::rng());
        Self { pieces, turns }
    }
}

impl BzPlayer {
    pub fn new() -> Self {
        Self {
            gameboard: empty_gameboard(),
            last_turn_played: 0,
        }
    }
}

impl Default for BzPlayer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_game(turns: u32) -> BzGame {
    BzGame::new(turns)
}

pub fn new_player() -> BzPlayer {
    BzPlayer::new()
}

pub fn place_piece(
    game: &BzGame,
    player: &BzPlayer,
    turn: u32,
    x: i32,
    y: i32,
) -> Result<BzGameboard, GameEngineError> {
    if turn == 0 {
        return Err(GameEngineError::InvalidTurn);
    }
    if turn > game.turns {
        return Err(GameEngineError::TurnExceedsGame);
    }
    if turn <= player.last_turn_played {
        return Err(GameEngineError::TurnAlreadyPlayed);
    }

    let piece = game
        .pieces
        .get((turn - 1) as usize)
        .ok_or(GameEngineError::TurnExceedsGame)?;
    validate_board(&player.gameboard)?;
    validate_piece(piece)?;

    let mut next_board = player.gameboard.clone();
    for (piece_y, row) in piece.iter().enumerate() {
        for (piece_x, &pip) in row.iter().enumerate() {
            if !pip {
                continue;
            }
            let board_x = x + piece_x as i32;
            let board_y = y + piece_y as i32;
            if board_x < 0 || board_x >= BOARD_SIZE as i32 || board_y < 0 || board_y >= BOARD_SIZE as i32 {
                return Err(GameEngineError::IllegalPlacement);
            }
            if next_board[board_y as usize][board_x as usize] {
                return Err(GameEngineError::IllegalPlacement);
            }
            next_board[board_y as usize][board_x as usize] = true;
        }
    }
    Ok(next_board)
}

pub fn is_legal_move(
    game: &BzGame,
    player: &BzPlayer,
    turn: u32,
    x: i32,
    y: i32,
) -> Result<BzGameboard, GameEngineError> {
    place_piece(game, player, turn, x, y)
}

pub fn compute_turn_score(
    game: &BzGame,
    player: &BzPlayer,
    turn: u32,
    board: BzGameboard,
) -> Result<TurnScore, GameEngineError> {
    validate_board(&board)?;
    if turn == 0 {
        return Err(GameEngineError::InvalidTurn);
    }
    if turn > game.turns {
        return Err(GameEngineError::TurnExceedsGame);
    }
    if turn <= player.last_turn_played {
        return Err(GameEngineError::TurnAlreadyPlayed);
    }

    let missed_turns = turn.saturating_sub(player.last_turn_played + 1);
    let (next_board, scored_blocks, block_score) = score_blocks(board);
    let mut score = block_score + FAILURE_PENALTY * missed_turns as i32;
    if turn == game.turns && next_board.iter().all(|row| row.iter().all(|&pip| !pip)) {
        score += FINAL_EMPTY_BOARD_BONUS;
    }

    Ok(TurnScore {
        score,
        gameboard: next_board,
        scored_blocks,
    })
}

pub fn compute_score(
    game: &BzGame,
    player: &BzPlayer,
    turn: u32,
    board: BzGameboard,
) -> Result<TurnScore, GameEngineError> {
    compute_turn_score(game, player, turn, board)
}

fn validate_board(board: &BzGameboard) -> Result<(), GameEngineError> {
    if board.len() != BOARD_SIZE || board.iter().any(|row| row.len() != BOARD_SIZE) {
        return Err(GameEngineError::InvalidBoard);
    }
    Ok(())
}

fn validate_piece(piece: &BzGamepiece) -> Result<(), GameEngineError> {
    if piece.len() != PIECE_SIZE || piece.iter().any(|row| row.len() != PIECE_SIZE) {
        return Err(GameEngineError::InvalidPiece);
    }
    Ok(())
}

fn score_blocks(mut board: BzGameboard) -> (BzGameboard, Vec<(usize, usize, usize, usize)>, i32) {
    let mut scored_blocks = Vec::new();
    let mut score = 0;
    for (width, height, points) in [(3, 3, 45), (3, 2, 15), (2, 3, 15), (2, 2, 5)] {
        for y in 0..=BOARD_SIZE - height {
            for x in 0..=BOARD_SIZE - width {
                if (y..y + height).all(|row| (x..x + width).all(|column| board[row][column])) {
                    for row in y..y + height {
                        for column in x..x + width {
                            board[row][column] = false;
                        }
                    }
                    scored_blocks.push((x, y, width, height));
                    score += points;
                }
            }
        }
    }
    (board, scored_blocks, score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_contains_each_piece_once() {
        let game = BzGame::new(31);
        let mut pieces = game.pieces.clone();
        pieces.sort_by_key(|piece| format!("{piece:?}"));
        let mut expected = all_gamepieces();
        expected.sort_by_key(|piece| format!("{piece:?}"));
        assert_eq!(pieces, expected);
    }

    #[test]
    fn placement_supports_empty_piece_edges() {
        let game = BzGame { pieces: vec![gamepiece_from_octal([0, 2, 0])], turns: 1 };
        let player = BzPlayer::new();
        assert!(place_piece(&game, &player, 1, -1, 0).is_ok());
        assert!(place_piece(&game, &player, 1, 0, 3).is_ok());
    }

    #[test]
    fn scoring_removes_a_full_block() {
        let game = BzGame { pieces: all_gamepieces(), turns: 2 };
        let player = BzPlayer::new();
        let board = vec![vec![true, true, true, false, false], vec![true, true, true, false, false], vec![true, true, true, false, false], vec![false; 5], vec![false; 5]];
        let result = compute_turn_score(&game, &player, 1, board).unwrap();
        assert_eq!(result.score, 45);
        assert!(result.gameboard.iter().flatten().all(|&pip| !pip));
    }

    #[test]
    fn invalid_overlap_and_turns_are_rejected() {
        let game = BzGame { pieces: vec![gamepiece_from_octal([0, 2, 0])], turns: 1 };
        let mut player = BzPlayer::new();
        player.gameboard[1][1] = true;
        assert_eq!(place_piece(&game, &player, 1, 0, 0), Err(GameEngineError::IllegalPlacement));
        assert_eq!(compute_turn_score(&game, &player, 2, empty_gameboard()), Err(GameEngineError::TurnExceedsGame));
    }

    #[test]
    fn missed_turns_are_penalized() {
        let game = BzGame { pieces: all_gamepieces(), turns: 3 };
        let player = BzPlayer::new();
        let result = compute_turn_score(&game, &player, 3, empty_gameboard()).unwrap();
        assert_eq!(result.score, -14 + FINAL_EMPTY_BOARD_BONUS);
    }
}