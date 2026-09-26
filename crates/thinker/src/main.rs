//! The live-model robot: reads the board each turn, asks the thinking model
//! which move to make, answers with one move word. Connection values come
//! only from the `.env` in the working directory and are never logged.

use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

const MOVES: [&str; 4] = ["left", "right", "rotate", "drop"];

const SYSTEM_PROMPT: &str = "You are an expert Tetris player. Board: 10 columns, 20 rows. \
After every word you answer, your piece slides down one row, so budget your answers: rotate to the \
best orientation, steer over the column where the piece's bottom contour matches the stack and fills \
a gap or completes a partial row (a full horizontal row of 10 clears and scores), then answer drop. \
Keep the stack flat; never leave holes under a piece. \
Reply with exactly one word: left, right, rotate, or drop. No other text.";

fn unquote(value: &str) -> &str {
    let mut v = value.trim();
    // Strip any nesting of plain and escaped quotes: "x", \"x\", "\"x\"" ...
    loop {
        if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
            v = v[1..v.len() - 1].trim();
        } else if v.len() >= 4 && v.starts_with("\\\"") && v.ends_with("\\\"") {
            v = v[2..v.len() - 2].trim();
        } else {
            break;
        }
    }
    v
}

fn load_env() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(".env") {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.trim().to_string(), unquote(value).to_string());
            }
        }
    }
    map
}

fn extract_move(text: &str) -> &'static str {
    for token in text.to_lowercase().split_whitespace() {
        let token = token.trim_matches(|c: char| !c.is_ascii_alphabetic());
        if MOVES.contains(&token) {
            match token {
                "left" => return "left",
                "right" => return "right",
                "rotate" => return "rotate",
                _ => return "drop",
            }
        }
    }
    "banana"
}

fn ask_model(
    agent: &ureq::Agent,
    env: &HashMap<String, String>,
    transcript: &str,
) -> Result<String, String> {
    let base = env.get("DEFAULT_BASE_URL").ok_or("DEFAULT_BASE_URL missing")?;
    let base = base.trim().trim_end_matches('/');
    let base = if base.contains("://") {
        base.to_string()
    } else {
        format!("https://{base}")
    };
    let key = env.get("DEFAULT_API_KEY").ok_or("DEFAULT_API_KEY missing")?;
    let model = env.get("DEFAULT_MODEL").ok_or("DEFAULT_MODEL missing")?;
    let api_version = env
        .get("DEFAULT_API_VERSION")
        .ok_or("DEFAULT_API_VERSION missing")?;

    // The base may be a bare resource root or a full deployment endpoint.
    let path = if base.contains("/deployments/") {
        "/chat/completions".to_string()
    } else {
        format!("/openai/deployments/{model}/chat/completions")
    };
    let url = format!("{base}{path}?api-version={api_version}");

    let body = serde_json::json!({
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": transcript}
        ],
        "temperature": 0.0,
        "max_tokens": 200
    });

    let response = agent
        .post(&url)
        .set("api-key", key)
        .send_json(body)
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("endpoint returned HTTP {code}"),
            _ => "could not reach the endpoint".to_string(),
        })?;
    let value: Value = response
        .into_json()
        .map_err(|e| format!("bad response body: {e}"))?;
    let content = value["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("no message content")?
        .to_string();
    Ok(content)
}

fn main() {
    let env = load_env();
    for key in [
        "DEFAULT_BASE_URL",
        "DEFAULT_API_KEY",
        "DEFAULT_MODEL",
        "DEFAULT_API_VERSION",
    ] {
        if !env.contains_key(key) {
            eprintln!("thinker: {key} missing from .env in the working directory");
            std::process::exit(1);
        }
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(45))
        .build();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let mut transcript = String::new();
    for line in BufReader::new(stdin.lock()).lines() {
        let Ok(line) = line else { break };
        transcript.push_str(&line);
        transcript.push('\n');
        if line.trim() != "board end" {
            continue;
        }

        let mut prompt = String::from(
            "Tetris board, 10 columns by 20 rows, top row first. \
             '#' is the stack, '@' is your current piece, '.' is empty.\n",
        );
        prompt.push_str(&transcript);
        prompt.push_str("Your move (one word: left, right, rotate, or drop):");

        let answer = match ask_model(&agent, &env, &prompt) {
            Ok(content) => extract_move(&content).to_string(),
            Err(err) => {
                eprintln!("thinker: {err}");
                "banana".to_string()
            }
        };
        if writeln!(out, "{answer}").is_err() {
            break;
        }
        let _ = out.flush();
        transcript.clear();
    }
}
