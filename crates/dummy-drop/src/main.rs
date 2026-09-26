//! A dummy robot that hard-drops every piece the moment it sees a board.

use std::io::{BufRead, BufReader, Write};

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for line in BufReader::new(stdin.lock()).lines() {
        let Ok(line) = line else { break };
        if line.trim() == "board end" {
            if writeln!(out, "drop").is_err() {
                break;
            }
            let _ = out.flush();
        }
    }
}
