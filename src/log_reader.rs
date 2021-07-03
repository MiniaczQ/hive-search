/*
Keep track of client status by reading the latest log file.
*/

use crate::client::*;
use regex::RegexSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::{Duration};

const LOG_CHECK_FREQUENCY: f32 = 1.0;

const STARTING: &str =
    r"\[..:..:..\] \[main/INFO\]: Started serving on ";
const STOPPING: &str =
    r"\[..:..:..\] \[Server thread/INFO\]: Stopping singleplayer server as player logged out";

const MATCHES: [&str; 2] = [STARTING, STOPPING];

enum MatchResult {
    StartedHosting(u16),
    StoppedHosting,
    Nothing,
}

/*
Matches a single line and returns the result.
*/
fn match_line(matcher: &RegexSet, line: &String) -> MatchResult {
    let matches = matcher.matches(line);
    if matches.matched(0) {
        let port = (&line[43..]).parse::<u16>().expect("Failed to parse port from log files.");
        return MatchResult::StartedHosting(port);
    } else if matches.matched(1) {
        return MatchResult::StoppedHosting;
    }
    MatchResult::Nothing
}

/*
Reads a single session of a client.
Session is defined as the minecraft client being open.
*/
fn read_session(matcher: &RegexSet, logs_path: &String, mut stream: TcpStream) {
    let mut last_size = 0u64;
    let mut last_position = 0u64;

    'outmost: loop {
        let file = File::open(logs_path).expect("Failed to open 'latest.log'.");
        let buf = &mut BufReader::new(file.try_clone().expect("Failed to clone file handle."));
        let size = file.metadata().expect("Failed to acquire file metadata.").len();

        if size != 0 {
            if size < last_size {
                let result = update_client_state(&mut stream, ClientStates::NotHosting);
                if let Err(_) = result {break 'outmost}
                last_position = 0;
            }
            last_size = size;

            buf.seek(SeekFrom::Start(last_position)).expect("Failed to seek a position in a file.");
            for line in buf.lines() {
                if let Ok(line) = line {
                    let result = match_line(matcher, &line);
                    match result {
                        MatchResult::StartedHosting(port) => {
                            let result = update_client_state(&mut stream, ClientStates::Hosting(port));
                            if let Err(_) = result {break 'outmost}
                        }
                        MatchResult::StoppedHosting => {
                            let result = update_client_state(&mut stream, ClientStates::NotHosting);
                            if let Err(_) = result {break 'outmost}
                        }
                        _ => {}
                    }
                }
            }
            last_position = buf.stream_position().expect("Failed to get the stream position of a file.");
        }

        sleep(Duration::from_secs_f32(LOG_CHECK_FREQUENCY));
    }
}

/*
Tracks logs and updates client state based on that.
*/
pub fn run(logs_path: String, stream: TcpStream) {
    let matcher = RegexSet::new(MATCHES).expect("Failed to create a RegexSet from static expressions.");
    read_session(&matcher, &logs_path, stream);
}
