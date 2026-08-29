use std::env;
use std::io::{self, BufRead, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    if env::args().nth(1).as_deref() == Some("--version") {
        println!("bench-observer-fixture 1.0");
        return Ok(());
    }

    if let Ok(delay_ms) = env::var("WORK_LEAF_OBSERVER_FIXTURE_USAGE_DELAY_MS") {
        return run_delayed_usage_app_server(delay_ms.parse().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid fixture usage delay: {error}"),
            )
        })?);
    }

    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    if let Some(stdout) = env::var_os("WORK_LEAF_OBSERVER_FIXTURE_STDOUT") {
        input = stdout.to_string_lossy().as_bytes().to_vec();
    }
    for chunk in input.chunks(5) {
        io::stdout().write_all(chunk)?;
        io::stdout().flush()?;
    }
    if let Some(stderr) = env::var_os("WORK_LEAF_OBSERVER_FIXTURE_STDERR") {
        io::stderr().write_all(stderr.to_string_lossy().as_bytes())?;
        io::stderr().flush()?;
    }
    if let Ok(code) = env::var("WORK_LEAF_OBSERVER_FIXTURE_EXIT_CODE")
        && let Ok(code) = code.parse::<i32>()
    {
        std::process::exit(code);
    }
    Ok(())
}

fn run_delayed_usage_app_server(delay_ms: u64) -> io::Result<()> {
    let (sender, receiver) = mpsc::channel::<String>();
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else {
                break;
            };
            if sender.send(line).is_err() {
                break;
            }
        }
    });

    let mut stdout = io::stdout().lock();
    let mut pending = None;
    loop {
        let line = match pending.take() {
            Some(line) => line,
            None => match receiver.recv() {
                Ok(line) => line,
                Err(_) => break,
            },
        };
        let value: serde_json::Value = serde_json::from_str(&line)?;
        match value.get("method").and_then(serde_json::Value::as_str) {
            Some("turn/start") => {
                let request_id = value["id"].clone();
                let thread_id = value["params"]["threadId"]
                    .as_str()
                    .unwrap_or("thread-grace");
                write_fixture_message(
                    &mut stdout,
                    &serde_json::json!({
                        "id": request_id,
                        "result": {"turn": {"id": "turn-grace"}},
                    }),
                )?;
                write_fixture_message(
                    &mut stdout,
                    &serde_json::json!({
                        "method": "thread/tokenUsage/updated",
                        "params": {
                            "threadId": thread_id,
                            "turnId": "turn-grace",
                            "tokenUsage": {
                                "total": {
                                    "inputTokens": 50,
                                    "cachedInputTokens": 40,
                                    "outputTokens": 5,
                                    "reasoningOutputTokens": 2,
                                },
                                "last": {
                                    "inputTokens": 50,
                                    "cachedInputTokens": 40,
                                    "outputTokens": 5,
                                    "reasoningOutputTokens": 2,
                                },
                            },
                        },
                    }),
                )?;
                write_fixture_message(
                    &mut stdout,
                    &serde_json::json!({
                        "method": "item/completed",
                        "params": {
                            "threadId": thread_id,
                            "turnId": "turn-grace",
                            "item": {
                                "type": "agentMessage",
                                "id": "message-grace",
                                "text": "@work-leaf done",
                            },
                        },
                    }),
                )?;
                if env::var_os("WORK_LEAF_OBSERVER_FIXTURE_OUTPUT_RESUMES").is_some() {
                    write_fixture_message(
                        &mut stdout,
                        &serde_json::json!({
                            "method": "item/started",
                            "params": {
                                "threadId": thread_id,
                                "turnId": "turn-grace",
                                "item": {
                                    "type": "reasoning",
                                    "id": "reasoning-after-directive",
                                },
                            },
                        }),
                    )?;
                }
                match receiver.recv_timeout(Duration::from_millis(delay_ms)) {
                    Ok(line) => pending = Some(line),
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        write_fixture_message(
                            &mut stdout,
                            &serde_json::json!({
                                "method": "thread/tokenUsage/updated",
                                "params": {
                                    "threadId": thread_id,
                                    "turnId": "turn-grace",
                                    "tokenUsage": {
                                        "total": {
                                            "inputTokens": 100,
                                            "cachedInputTokens": 80,
                                            "outputTokens": 10,
                                            "reasoningOutputTokens": 5,
                                        },
                                        "last": {
                                            "inputTokens": 100,
                                            "cachedInputTokens": 80,
                                            "outputTokens": 10,
                                            "reasoningOutputTokens": 5,
                                        },
                                    },
                                },
                            }),
                        )?;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            Some("turn/interrupt") => {
                let request_id = value["id"].clone();
                let thread_id = value["params"]["threadId"]
                    .as_str()
                    .unwrap_or("thread-grace");
                write_fixture_message(
                    &mut stdout,
                    &serde_json::json!({
                        "method": "turn/completed",
                        "params": {
                            "threadId": thread_id,
                            "turn": {"id": "turn-grace", "status": "interrupted"},
                        },
                    }),
                )?;
                write_fixture_message(
                    &mut stdout,
                    &serde_json::json!({"id": request_id, "result": {}}),
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn write_fixture_message(output: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    output.write_all(b"\n")?;
    output.flush()
}
