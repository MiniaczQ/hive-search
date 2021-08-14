/*
Logs reader.
*/

use async_std::{channel::{Receiver, Sender}, task::sleep};
use futures::{FutureExt, pin_mut, select};
use regex::RegexSet;
use std::{fs::File, sync::Arc};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::time::Duration;

use crate::sync::PauseToken;

const STARTING: &str = r"\[..:..:..\] \[main/INFO\]: Started serving on ";
const STOPPING: &str =
    r"\[..:..:..\] \[Server thread/INFO\]: Stopping singleplayer server as player logged out";

pub enum ClientChange {
    StartedHosting(u16),
    StoppedHosting,
}

/// Regularly polls logs for updates.
/// Waits `duration` between polls.
/// New durations can be sent through `durations_recv`.
/// Can be paused and stopped with tokens.
/// Scans only the provided path.
/// Signals about prompts to `log_sink`.
pub async fn log_reader(
    stop_token: Arc<PauseToken>,
    pause_token: Arc<PauseToken>,
    mut duration: Duration,
    durations_recv: Receiver<Duration>,
    log_path: String,
    log_sink: Sender<ClientChange>,
) {
    let mut memory = LogPollMemory::new();
    let matcher = RegexSet::new([STARTING, STOPPING]).unwrap();
    while stop_token.is_paused().await {
        let delay = sleep(duration.clone()).fuse();
        let new_duration = durations_recv.recv().fuse();
        let stop = stop_token.wait().fuse();
        pin_mut!(delay);
        pin_mut!(new_duration);
        pin_mut!(stop);

        select! {
            _ = delay => {},
            result = new_duration => {
                if let Ok(new_duration) = result {
                    duration = new_duration;
                } else {
                    break
                }
            },
            _ = stop => break,
        }

        poll_logs(&mut memory, &matcher, &log_path, &log_sink).await;

        pause_token.wait().await;
    }
}

/// Stores data to detect updates in log files
struct LogPollMemory {
    last_position: u64,
    last_size: u64,
}

impl LogPollMemory {
    fn new() -> Self {
        Self {
            last_position: 0,
            last_size: 0,
        }
    }
}

/// Checks for updates in logs.
async fn poll_logs(
    memory: &mut LogPollMemory,
    matcher: &RegexSet,
    log_path: &String,
    log_sink: &Sender<ClientChange>,
) {
    let result = File::open(log_path);
    if let Ok(logs_file) = result {
        let size = logs_file.metadata().unwrap().len();
        let logs = &mut BufReader::new(logs_file);
        if size != 0 {
            if size < memory.last_size {
                let _ = log_sink.send(ClientChange::StoppedHosting).await;
                memory.last_size = 0;
            }
            memory.last_size = size;
            logs.seek(SeekFrom::Start(memory.last_position)).unwrap();
            if let Some(change) = match_lines(&matcher, logs) {
                let _ = log_sink.send(change).await;
            }
            memory.last_position = logs.stream_position().unwrap();
        }
    }
}

/// Scans all newly added lines.
/// Returns only the latest change.
fn match_lines(
    matcher: &RegexSet,
    buffer: &mut BufReader<File>,
) -> Option<ClientChange> {
    let mut final_result: Option<ClientChange> = None;
    for result in buffer.lines() {
        if let Ok(line) = result {
            let result = match_line(matcher, line);
            if let Some(change) = result {
                final_result = Some(change);
            }
        }
    }
    final_result
}

/// Scans a single line for a prompt.
fn match_line(
    matcher: &RegexSet,
    line: String,
) -> Option<ClientChange> {
    let matches = matcher.matches(&line);
    if matches.matched(0) {
        let port = (&line[43..])
            .parse::<u16>()
            .expect("Failed to parse port from log files.");
        return Some(ClientChange::StartedHosting(port));
    } else if matches.matched(1) {
        return Some(ClientChange::StoppedHosting);
    }
    None
}