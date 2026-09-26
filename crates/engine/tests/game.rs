use engine::{run_game, GameConfig, GameEnd};

fn run(cmd: &str, timer: Option<u64>) -> (engine::GameResult, String) {
    let cfg = GameConfig {
        seed: 42,
        player_cmd: cmd.to_string(),
        timer,
    };
    let mut buf: Vec<u8> = Vec::new();
    let result = run_game(&cfg, &mut buf);
    let log = String::from_utf8(buf).unwrap();
    (result, log)
}

#[test]
fn nonsense_answers_still_place_pieces_and_never_accept() {
    let (result, log) = run("yes banana", None);
    assert_eq!(result.end, GameEnd::TopOut);
    assert!(result.pieces > 0);
    assert_eq!(result.points, 0);
    assert_eq!(result.lines, 0);
    assert!(log.contains("rejected banana"));
    assert!(log.contains("placed "));
    assert!(!log.contains("accepted"));
    assert!(log.contains("result points="));
}

#[test]
fn blocked_moves_at_the_wall_are_rejected() {
    let (result, log) = run("yes left", None);
    assert_eq!(result.end, GameEnd::TopOut);
    assert!(log.contains("rejected left"));
    assert!(log.contains("accepted left"));
    assert!(log.contains("result points="));
}

#[test]
fn dropper_ends_by_itself_at_top_out() {
    let (result, log) = run("yes drop", None);
    assert_eq!(result.end, GameEnd::TopOut);
    assert!(result.pieces > 0);
    assert!(log.contains("accepted drop"));
    assert!(log.contains("result points="));
}

#[test]
fn same_seed_same_log() {
    let (_, a) = run("yes banana", None);
    let (_, b) = run("yes banana", None);
    assert_eq!(a, b);
}

#[test]
fn board_block_is_twenty_rows_of_ten_cells() {
    let (_, log) = run("yes drop", None);
    let lines: Vec<&str> = log.lines().collect();
    let begin = lines.iter().position(|l| *l == "board begin").unwrap();
    let end = lines.iter().position(|l| *l == "board end").unwrap();
    assert_eq!(end - begin - 1, 20);
    for row in &lines[begin + 1..end] {
        assert_eq!(row.len(), 10);
    }
}

#[test]
fn timer_off_records_no_nudges_for_slow_robot() {
    let (_, log) = run("while true; do sleep 1; echo drop; done", None);
    assert!(!log.contains("timer nudge"));
    assert!(log.contains("result points="));
}

#[test]
fn timer_on_nudges_silent_seconds() {
    let (_, log) = run("while true; do sleep 2; echo drop; done", Some(1));
    assert!(log.contains("timer nudge"));
    assert!(log.contains("result points="));
}

#[test]
fn board_driven_robot_gets_a_board_for_every_answer() {
    // This robot only answers after it sees a board: it hard-drops forever.
    let robot = "while read -r line; do if [ \"$line\" = \"board end\" ]; then echo drop; fi; done";
    let (result, log) = run(robot, None);
    assert_eq!(result.end, GameEnd::TopOut);
    assert!(result.pieces > 0);
    assert!(log.contains("accepted drop"));
    assert!(log.contains("result points="));
}

#[test]
fn quitting_robot_keeps_score_so_far() {
    // Drops one piece then quits immediately.
    let (result, log) = run("echo drop; sleep 10", None);
    assert_eq!(result.end, GameEnd::PlayerExit);
    assert!(result.pieces >= 1);
    assert!(log.contains("result points="));
}

#[test]
fn write_trait_object_is_used() {
    // Referee runs games with a silent log; make sure a generic Write works.
    let cfg = GameConfig {
        seed: 7,
        player_cmd: "yes drop".to_string(),
        timer: None,
    };
    let mut sink = std::io::sink();
    let result = run_game(&cfg, &mut sink);
    assert!(result.pieces > 0);
}
