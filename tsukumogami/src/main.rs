use tsukumogami::dispatch;

use std::io::{self, BufRead, Write};

use protocol::{Request, Response};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("gami: read error: {e}");
                break;
            }
        };
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Err(e) => Response::err("?", format!("parse error: {e}")),
            Ok(req) => dispatch::dispatch(req),
        };

        let mut serialized = serde_json::to_string(&response).expect("response serialize");
        serialized.push('\n');
        out.write_all(serialized.as_bytes()).expect("write");
        out.flush().expect("flush");
    }
}
