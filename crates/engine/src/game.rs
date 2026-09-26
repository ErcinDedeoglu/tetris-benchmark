//! The game loop: a robot program joins over the line protocol and plays one
//! seeded game to top-out.

use crate::board::{score_for_lines, Board, WIDTH};
use crate::piece::{Piece, SHAPE_NAMES};
use crate::prng::Rng;
use crate::robot::{Answer, RobotSession};
use std::io::Write;

pub const MOVE_WORDS: [&str; 4] = ["left", "right", "rotate", "drop"];

#[derive(Clone)]
pub struct GameConfig {
    pub seed: u64,
    pub player_cmd: String,
    pub timer: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEnd {
    TopOut,
    PlayerExit,
    OutputClosed,
}

/// The game's echo stream. Board blocks, accepted moves, placements and
/// nudges are written live; `rejected` answers are collected and flushed as
/// one block right before the final `result` line, so a `grep -m 1` watching
/// the stream for a rejection cannot close the pipe before the scoreboard
/// line has been written. A broken output stream ends the game quietly.
struct Echo<'a> {
    inner: &'a mut dyn Write,
    broken: bool,
    deferred_rejects: Vec<String>,
}

impl<'a> Echo<'a> {
    fn new(inner: &'a mut dyn Write) -> Self {
        Echo {
            inner,
            broken: false,
            deferred_rejects: Vec::new(),
        }
    }

    fn line(&mut self, text: &str) {
        if self.broken {
            return;
        }
        if writeln!(self.inner, "{text}").is_err() {
            self.broken = true;
        }
    }

    fn block(&mut self, text: &str) {
        if self.broken {
            return;
        }
        if self.inner.write_all(text.as_bytes()).is_err() {
            self.broken = true;
        }
    }

    fn reject(&mut self, word: &str) {
        self.deferred_rejects.push(format!("rejected {word}"));
    }

    fn tail(&mut self, over: &str) {
        self.line(over);
        for rejected in std::mem::take(&mut self.deferred_rejects) {
            self.line(&rejected);
        }
    }
}

pub struct GameResult {
    pub points: u64,
    pub lines: u64,
    pub pieces: u64,
    pub end: GameEnd,
}

fn lock_and_score(
    board: &mut Board,
    piece: &Piece,
    echo: &mut Echo,
    pieces: &mut u64,
    lines_total: &mut u64,
    points: &mut u64,
) {
    board.set_cells(&piece.cells());
    *pieces += 1;
    echo.line(&format!("placed {}", piece.shape_name()));
    let cleared = board.clear_full_rows();
    if cleared > 0 {
        echo.line(&format!("cleared {}", cleared));
        *lines_total += cleared as u64;
        *points += score_for_lines(cleared);
    }
}

/// Run one full game. All exchanged lines are echoed to `log`.
pub fn run_game(cfg: &GameConfig, log: &mut dyn Write) -> GameResult {
    let mut echo = Echo::new(log);
    echo.line(&format!("seed {}", cfg.seed));
    echo.line(&format!("player {}", cfg.player_cmd));

    let mut robot = match RobotSession::join(&cfg.player_cmd) {
        Ok(r) => r,
        Err(err) => {
            echo.tail(&format!("game over player-exit ({err})"));
            echo.line("result points=0 lines=0 pieces=0");
            return GameResult {
                points: 0,
                lines: 0,
                pieces: 0,
                end: GameEnd::PlayerExit,
            };
        }
    };

    let mut board = Board::new();
    let mut rng = Rng::new(cfg.seed);
    let mut points = 0u64;
    let mut lines_total = 0u64;
    let mut pieces = 0u64;

    let mut active: Option<Piece> = None;
    let outcome = loop {
        // Spawn the next piece when none is in play; the stack reaching the
        // spawn cells ends the game.
        if active.is_none() {
            let shape = (rng.next_u64() % SHAPE_NAMES.len() as u64) as usize;
            let piece = Piece::spawn(shape, WIDTH as i32);
            if !board.cells_free(&piece.cells()) {
                break GameEnd::TopOut;
            }
            active = Some(piece);
        }

        // Send the board state and wait for one answer (or a timer nudge).
        let current = active.as_ref().expect("piece in play");
        let state = board.render(current.shape_name(), &current.cells());
        robot.send(&state);
        echo.block(&state);
        if echo.broken {
            break GameEnd::OutputClosed;
        }

        let answer = match robot.read_answer(cfg.timer) {
            Answer::Word(w) => Some(w),
            Answer::Timeout => {
                echo.line("timer nudge");
                None
            }
            Answer::Disconnected => break GameEnd::PlayerExit,
        };

        let word = match answer {
            Some(w) => w.trim().to_string(),
            None => String::new(),
        };

        if !word.is_empty() {
            let piece = active.as_mut().expect("piece in play");
            let mut accepted = false;
            match word.as_str() {
                "left" => {
                    let mut shifted = *piece;
                    shifted.col -= 1;
                    if board.cells_free(&shifted.cells()) {
                        *piece = shifted;
                        accepted = true;
                    }
                }
                "right" => {
                    let mut shifted = *piece;
                    shifted.col += 1;
                    if board.cells_free(&shifted.cells()) {
                        *piece = shifted;
                        accepted = true;
                    }
                }
                "rotate" => {
                    let mut turned = *piece;
                    turned.state = (turned.state + 1) % 4;
                    if board.cells_free(&turned.cells()) {
                        *piece = turned;
                        accepted = true;
                    }
                }
                "drop" => {
                    let mut dropped = *piece;
                    loop {
                        let mut lower = dropped;
                        lower.row += 1;
                        if board.cells_free(&lower.cells()) {
                            dropped = lower;
                        } else {
                            break;
                        }
                    }
                    *piece = dropped;
                    accepted = true;
                }
                _ => {}
            }
            if accepted {
                echo.line(&format!("accepted {word}"));
            } else {
                echo.reject(&word);
            }

            if accepted && word == "drop" {
                let locked = active.take().expect("piece in play");
                lock_and_score(
                    &mut board,
                    &locked,
                    &mut echo,
                    &mut pieces,
                    &mut lines_total,
                    &mut points,
                );
                continue;
            }
        }

        // After every answer (and after every nudge) the active piece slides
        // down one row; if that is blocked it lands.
        let piece = active.as_mut().expect("piece in play");
        let mut lower = *piece;
        lower.row += 1;
        if board.cells_free(&lower.cells()) {
            *piece = lower;
        } else {
            let locked = active.take().expect("piece in play");
            lock_and_score(
                &mut board,
                &locked,
                &mut echo,
                &mut pieces,
                &mut lines_total,
                &mut points,
            );
        }

        if echo.broken {
            break GameEnd::OutputClosed;
        }
    };

    let over = match outcome {
        GameEnd::TopOut => "game over top-out",
        GameEnd::PlayerExit => "game over player-exit",
        GameEnd::OutputClosed => "game over output-closed",
    };
    echo.tail(over);
    echo.line(&format!(
        "result points={} lines={} pieces={}",
        points, lines_total, pieces
    ));
    robot.close();
    GameResult {
        points,
        lines: lines_total,
        pieces,
        end: outcome,
    }
}
