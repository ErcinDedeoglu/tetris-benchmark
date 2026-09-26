use engine::{run_game, GameConfig};
use std::io::{BufWriter, Write};

fn usage() -> ! {
    eprintln!("usage: tetris [--seed N] --player CMD [--timer SECONDS]");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: u64 = 42;
    let mut player = String::new();
    let mut timer: Option<u64> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                i += 1;
                seed = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            "--player" => {
                i += 1;
                player = args.get(i).cloned().unwrap_or_else(|| usage());
            }
            "--timer" => {
                i += 1;
                timer = Some(
                    args.get(i)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or_else(|| usage()),
                );
            }
            _ => usage(),
        }
        i += 1;
    }
    if player.is_empty() {
        usage();
    }

    let cfg = GameConfig {
        seed,
        player_cmd: player,
        timer,
    };
    // The whole game log is flushed in one burst at the end so a
    // `cmd | tee file | grep -m 1 ...` watcher can never close the pipe
    // before the final result line has been copied into the file.
    let stdout = std::io::stdout();
    let mut out = BufWriter::with_capacity(1024 * 1024, stdout.lock());
    let _ = run_game(&cfg, &mut out);
    let _ = out.flush();
}
