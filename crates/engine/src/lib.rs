//! Tetris engine: seeded games played by external robot programs over a
//! plain-text line protocol.

pub mod board;
pub mod game;
pub mod piece;
pub mod prng;
pub mod robot;

pub use board::{Board, HEIGHT, WIDTH};
pub use game::{run_game, GameConfig, GameEnd, GameResult, MOVE_WORDS};
pub use piece::{Piece, SHAPE_NAMES};
pub use prng::Rng;
pub use robot::{Answer, RobotSession};
