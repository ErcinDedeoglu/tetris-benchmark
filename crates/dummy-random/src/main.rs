//! A dummy robot that answers with a pseudo-random move word each turn.
//! Its choices are derived from the game seed alone: the FNV-1a hash of the
//! board lines it has seen, so the same seed replays the same choices.

use std::io::{BufRead, BufReader, Write};

const MOVES: [&str; 4] = ["left", "right", "rotate", "drop"];

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for line in BufReader::new(stdin.lock()).lines() {
        let Ok(line) = line else { break };
        for &b in line.as_bytes() {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= u64::from(b'\n');
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        if line.trim() == "board end" {
            let mixed = hash ^ (hash >> 29);
            let mv = MOVES[(mixed % MOVES.len() as u64) as usize];
            if writeln!(out, "{mv}").is_err() {
                break;
            }
            let _ = out.flush();
        }
    }
}
