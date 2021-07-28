/*
Logs reader.
*/

use regex::RegexSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SendError, Sender};
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;

use crate::message::client::ClientMessage;

const LOG_CHECK_FREQUENCY: f32 = 5.0;

const STARTING: &str = r"\[..:..:..\] \[main/INFO\]: Started serving on ";
const STOPPING: &str =
    r"\[..:..:..\] \[Server thread/INFO\]: Stopping singleplayer server as player logged out";

pub enum ClientChange {
    StartedHosting(u16),
    StoppedHosting,
}

/*
Returns the regex set matcher.
*/
fn create_matcher() -> RegexSet {
    RegexSet::new([STARTING, STOPPING])
        .expect("Failed to create a RegexSet from static expressions.")
}

/*
Matches a single line and returns the result.
*/
fn match_line(matcher: &RegexSet, line: &String) -> Option<ClientChange> {
    let matches = matcher.matches(line);
    if matches.matched(0) {
        let port = (&line[43..])
            .parse::<u16>()
            .expect("Failed to parse port from log files.");
        return Some(ClientChange::StartedHosting(port))
    } else if matches.matched(1) {
        return Some(ClientChange::StoppedHosting)
    }
    None
}

/*
Match lines until end of file.
Return the latest result.
*/
fn match_lines(matcher: &RegexSet, buffer: &mut BufReader<File>) -> Option<ClientChange> {
    let mut result: Option<ClientChange> = None;
    for line in buffer.lines() {
        if let Ok(line) = line {
            let hit = match_line(matcher, &line);
            if let Some(_) = hit {
                result = hit;
            }
        }
    }
    result
}

/*
Send message to client.
*/
fn send(client_sink: &Sender<ClientMessage>, change: ClientChange) -> Result<(), SendError<ClientMessage>> {
    let message: ClientMessage;
    match change {
        ClientChange::StartedHosting(port) => {
            message = ClientMessage::StartedHosting(port);
        },
        ClientChange::StoppedHosting => {
            message = ClientMessage::StoppedHosting;
        }
    }
    client_sink.send(message)
}

/*
Runs the functionality.
*/
fn run(
    log_path: String,
    client_sink: Sender<ClientMessage>,
    breaker: Arc<AtomicBool>,
) {
    let mut last_size: u64 = 0;
    let mut last_position: u64 = 0;
    let matcher = create_matcher();

    while breaker.load(Ordering::Relaxed) {
        let file = File::open(&log_path).expect("Failed to open 'latest.log'.");
        let size = file
            .metadata()
            .expect("Failed to acquire file metadata.")
            .len();
        let buffer = &mut BufReader::new(file);

        let mut result: Option<ClientChange> = None;
        if size != 0 {
            if size < last_size {
                result = Some(ClientChange::StoppedHosting);
                last_position = 0;
            }
            last_size = size;
            buffer.seek(SeekFrom::Start(last_position))
                .expect("Failed to seek a position in a file.");

            let last_hit = match_lines(&matcher, buffer);
            if let Some(_) = last_hit {
                result = last_hit;
            }

            last_position = buffer.stream_position()
                .expect("Failed to get the stream position.");
        }
        
        if let Some(change) = result {
            if let Err(_) = send(&client_sink, change) {
                break;
            }
        }

        sleep(Duration::from_secs_f32(LOG_CHECK_FREQUENCY));
    }
}

/*
Start the functionality in another thread.
Returns handle.
*/
pub fn spawn(
    log_path: String,
    client_sink: Sender<ClientMessage>,
    breaker: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Client Log Reader".to_string())
        .spawn(move || run(log_path, client_sink, breaker))
        .expect("Failed to start the Client Log Reader thread.")
}
