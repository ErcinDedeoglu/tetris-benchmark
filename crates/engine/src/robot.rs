//! A joined robot: a separate OS program spoken to over newline-delimited
//! plain text on stdin/stdout.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

pub enum Answer {
    /// The robot answered with a line.
    Word(String),
    /// The timer elapsed with no answer.
    Timeout,
    /// The robot quit or stopped answering.
    Disconnected,
}

pub struct RobotSession {
    child: Child,
    to_robot: Sender<String>,
    from_robot: Receiver<String>,
}

impl RobotSession {
    /// Join a robot by running `cmd` through `sh -c`.
    pub fn join(cmd: &str) -> std::io::Result<Self> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");

        // Writer thread: boards go out without ever blocking the game loop,
        // even when the robot never reads its stdin (e.g. `yes banana`).
        let (to_robot, writer_rx) = mpsc::channel::<String>();
        thread::spawn(move || {
            let mut stdin = stdin;
            while let Ok(text) = writer_rx.recv() {
                if stdin.write_all(text.as_bytes()).is_err() {
                    break;
                }
                let _ = stdin.flush();
            }
        });

        // Reader thread: one line at a time from the robot's stdout.
        let (reader_tx, from_robot) = mpsc::channel::<String>();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if reader_tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(RobotSession {
            child,
            to_robot,
            from_robot,
        })
    }

    /// Send a text block (board state) to the robot.
    pub fn send(&self, text: &str) {
        let _ = self.to_robot.send(text.to_string());
    }

    /// Wait for the robot's next answer line, honoring the optional timer.
    pub fn read_answer(&self, timer: Option<u64>) -> Answer {
        match timer {
            None => match self.from_robot.recv() {
                Ok(line) => Answer::Word(line),
                Err(_) => Answer::Disconnected,
            },
            Some(secs) => match self.from_robot.recv_timeout(Duration::from_secs(secs)) {
                Ok(line) => Answer::Word(line),
                Err(RecvTimeoutError::Timeout) => Answer::Timeout,
                Err(RecvTimeoutError::Disconnected) => Answer::Disconnected,
            },
        }
    }

    pub fn close(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
