//! A2S relay: HTTP GET /status -> one A2S_INFO + A2S_PLAYER query (UDP) -> JSON.

use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use a2s::A2SClient;
use serde::Serialize;

#[derive(Serialize, Clone)]
struct Player {
    /// Rust returns anonymised names that change on every reconnect.
    name: String,
    duration_secs: u32,
}

#[derive(Serialize, Clone)]
struct Status {
    ok: bool,
    error: Option<String>,
    name: String,
    map: String,
    players: u8,
    max_players: u8,
    bots: u8,
    player_list: Vec<Player>,
    fetched_at: u64,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn query(addr: &str) -> Status {
    let base = Status {
        ok: false,
        error: None,
        name: String::new(),
        map: String::new(),
        players: 0,
        max_players: 0,
        bots: 0,
        player_list: vec![],
        fetched_at: now_unix(),
    };
    let result = (|| -> Result<Status, String> {
        let mut client = A2SClient::new().map_err(|e| format!("socket: {e}"))?;
        client
            .set_timeout(Duration::from_secs(3))
            .map_err(|e| format!("set_timeout: {e}"))?;
        let info = client.info(addr).map_err(|e| format!("A2S_INFO: {e}"))?;
        // Player list is best-effort; the count from A2S_INFO is authoritative.
        let player_list = client
            .players(addr)
            .map(|ps| {
                ps.into_iter()
                    .map(|p| Player {
                        name: p.name,
                        duration_secs: p.duration.max(0.0) as u32,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Status {
            ok: true,
            name: info.name,
            map: info.map,
            players: info.players,
            max_players: info.max_players,
            bots: info.bots,
            player_list,
            ..base.clone()
        })
    })();
    result.unwrap_or_else(|e| Status {
        error: Some(e),
        ..base
    })
}

/// inetd style: systemd socket activation (Accept=yes) hands one HTTP
/// connection to stdin/stdout. We answer once and exit — no resident process.
fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "153.120.121.159:28016".into());

    let mut stdin = BufReader::new(std::io::stdin().lock());
    let mut request_line = String::new();
    let _ = stdin.read_line(&mut request_line);
    // Drain headers up to the blank line.
    let mut line = String::new();
    while stdin.read_line(&mut line).map(|n| n > 0).unwrap_or(false) && line.trim_end() != "" {
        line.clear();
    }

    let path = request_line.split_whitespace().nth(1).unwrap_or("");
    let (status, body) = if path == "/status" {
        // No cache: each page load / refresh is exactly one query.
        ("200 OK", serde_json::to_string(&query(&target)).unwrap())
    } else {
        ("404 Not Found", r#"{"ok":false,"error":"not found"}"#.to_string())
    };

    let mut out = std::io::stdout().lock();
    let _ = write!(
        out,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = out.flush();
}
