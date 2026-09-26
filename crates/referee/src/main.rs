use engine::{run_game, GameConfig};

struct Totals {
    name: String,
    points: u64,
    lines: u64,
    pieces: u64,
}

fn player_name(cmd: &str) -> String {
    let first = cmd.split_whitespace().next().unwrap_or("player");
    first.rsplit('/').next().unwrap_or(first).to_string()
}

fn usage() -> ! {
    eprintln!(
        "usage: referee [--seed N] [--games N] --players CMD,CMD,... [--timer SECONDS]"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: u64 = 42;
    let mut games: u64 = 3;
    let mut players_arg = String::new();
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
            "--games" => {
                i += 1;
                games = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            "--players" => {
                i += 1;
                players_arg = args.get(i).cloned().unwrap_or_else(|| usage());
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

    let players: Vec<String> = players_arg
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if players.is_empty() {
        usage();
    }

    let mut totals: Vec<Totals> = Vec::new();
    for cmd in &players {
        let name = player_name(cmd);
        let (mut points, mut lines, mut pieces) = (0u64, 0u64, 0u64);
        for game in 0..games {
            // Every robot faces the same seeded piece sequences per game.
            let cfg = GameConfig {
                seed: seed + game,
                player_cmd: cmd.clone(),
                timer,
            };
            let mut sink = std::io::sink();
            let res = run_game(&cfg, &mut sink);
            println!(
                "{} game {}: points={} lines={} pieces={}",
                name,
                game + 1,
                res.points,
                res.lines,
                res.pieces
            );
            points += res.points;
            lines += res.lines;
            pieces += res.pieces;
        }
        totals.push(Totals {
            name,
            points,
            lines,
            pieces,
        });
    }

    // Rank by points only, descending; ties share a rank.
    totals.sort_by(|a, b| b.points.cmp(&a.points).then(a.name.cmp(&b.name)));

    let mut board = String::new();
    board.push_str(&format!(
        "# Tetris benchmark scoreboard (seed {}, {} games per player)\n",
        seed, games
    ));
    board.push('\n');
    board.push_str("| rank | player | points | lines | pieces |\n");
    board.push_str("| --- | --- | --- | --- | --- |\n");
    let mut rank = 0usize;
    for (i, t) in totals.iter().enumerate() {
        if i == 0 || t.points != totals[i - 1].points {
            rank = i + 1;
        }
        board.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            rank, t.name, t.points, t.lines, t.pieces
        ));
    }

    std::fs::write("scoreboard.md", &board).expect("write scoreboard.md");
    print!("{board}");
    println!("scoreboard written to scoreboard.md");
}
