---
id: "01-game-engine"
title: "Implement game engine structs and turn operations"
status: "queued"
priority: 2
depends_on: []
skills_required:
  - ".agents/skills/rocket.md"
  - ".agents/skills/surrealdb.md"
verification:
  - "cargo check --bin backend"
  - "cargo test --test ge_tests"
---

### Objective

Implement rust structures and rules for playing the game. Implement only game engine functions;
do not implement persistence, turn tracking, time tracking, or player management functions (e.g:
adding players to a game, tracking player history)

### Overview of the game

The goal of the game is to achieve a high score. At each turn, a randomly selected 3x3 game piece is
presented to the player. The player has a private 5x5 game board for the duration of the game. The
player is expected to place the game piece on the board in a way that it does not conflict with
pips from previously placed game pieces. The player cannot alter the orientation of the game piece
or the game board.

The player wants to strategically place the piece such that it will form blocks of pips. During the
reward stage of each turn, if the player has created a new 3x3, 3x2, 2x3, or 2x2 set of contiguous
pips, some amount will be added to the players total score, then those pips are removed from the
game board to be ready for the next turn.

Each game piece will be a 3x3 array of empty spaces and pips. To place a game piece on the board,
the player must find a position for the piece such that no pips from the game piece conflict with
the pips previously placed on the board.

Scoring is as follows for each turn:

1) 3x3 block - 45 pts
1) 3x2 or 2x3 block - 15 pts
1) 2x3 block - 5 pts
1) failure to place the game piece - -7 pts (negative)

At the beginning of the game, all players' game boards are empty. Each player makes their own
decisions where to place each game piece. Good strategy will include positioning the current piece
such that it does not make any blocks in hopes that a near future turn will bring a piece that
can be placed for the maximum score.

Note that the values for the block reward scores could change in future versions

### Object representation

A game board can be represented by Vec of length 5 for the 5 rows. Each element could be represented
by a Vec of 5 bools. If the bool is true, it would indicate a pip -- the corresponding space on
the game board is occupied.

A game piece can be represented by Vec of length 3 for the 3 rows. Each element could be 
represented by a Vec of 3 bools.

The orientation of the game board and pieces should be that row 0 is the topmost row. In each row
Vec, the 0th element is the rightmost. This primarily matters in the user interface which will be
defined in a future prompt.

### Type naming

The game board should be a type named BzGameboard. The game piece should be a type named BzGamepiece

### Set of game pieces

To be concise, the pips in a game piece can be represented by 3 octal numbers. The bits in each
octal number correspond to the pips in the piece. For example 0,7,0 would have top row empty, middle
row all pips, and bottom row empty. A vertical line could be represented by 2,2,2.

This is the list of the possible game pieces (31 different pieces):

0,2,0
2,2,0
0,3,0
0,7,0
2,2,2
1,2,0
4,2,0
1,2,4
4,2,1

2,3,0
0,3,2
0,6,2
2,6,0

1,7,0
0,7,1
4,7,0
0,7,4
3,2,2
6,2,2
2,2,3
2,2,6

2,3,2
0,7,2
2,6,2
2,7,0
3,2,3
0,7,5
6,2,6
5,7,0

5,2,5
2,7,2

### Multiplayer game play

When a game is created, all player's game boards will be empty and all player's scores will be zero.
The system chooses the order of the game pieces randomly but does not reveal the order to players.
When the first turn starts, the system shows all players the game piece for the turn. Players have
a limited amount of time to place the piece on the game board. Any player that does not place the
turn's game piece on the board is penalized 7 points.

When time for the turn has expired, each game board is evaluated for contiguous blocks. Note that
is possible for a single turn's move to create multiple blocks that score. The player's turn score
is added to the player's total score and the scored blocks are removed from that player's game
board.

All players will have the same time limit for each turn.

### Implementation notes

The game engine does not need to manage time. That will come from a management controller to be
defined later. The game engine should include these operations:

- Create a new game. The game struct should be named BzGame.
- BzGame should include a randomly ordered Vec of all possible game pieces.
- BzGame should include the number of turns in the game.
- Create a player. The player struct should be named BzPlayer.
- The player struct should include a BzGameboard.
- BzPlayer should also include the turn number of the last turn played
- Compute score for a player's turn given a game board and a turn number
  - Find contiguous blocks
  - compute score for turn
  - compute resulting game board with scored blocks removed
  - when turn number is less than last turn played, it is an error
  - when turn number is 1 more than last turn played, score normally
  - when turn number is more than last turn played + 1, include penalties in the computed score
- Compute whether a player's move is legal
  - function will take parameters for BzGame, BzPlayer, turn number, and (x, y) placement
  - a game piece positioned on the game board at (x, y) that conflicts with any pips, is an error
  - if the game piece fits on the game board at (x, y) compute the resulting game board
- A game piece that has no pips in the top row can be placed at y = -1
- A game piece that has no pips in the bottom row can be placed at y = 3
- A game piece that has no pips in the rightmost column can be placed at x = -1
- A game piece that has no pips in the leftmost column can be placed at x = 3
- An empty game board on the final turn of the game gets an extra bonus of 15 (3x2 reward)

It is not necessary for the game engine to track time or turns. The management controller and
persistence layer will be responsible for that. The game engine just computes turn score and
next game board.

### Tests

- Ensure that a newly initialized game includes each game piece exactly one time
- Set up five different situations for game piece placement during a game and validate the result
- Set up some situations that will result in an error and validate the result
- Test the condition where the turn number exceeds the number of turns in the game
