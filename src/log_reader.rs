/*
Keep track of client status by reading the latest log file.
*/

use regex::{RegexSet};
use std::fs::{File};
use std::io::{BufRead, BufReader};
use std::time::{Duration, SystemTime};
use std::thread::sleep;

use crate::client_state::*;

const LOG_CHECK_FREQUENCY: (u64, u32) = (2, 500_00_00);

const STARTING: &str = r"\[..:..:..\] \[main/INFO\]: Started serving on "; // Ends with the port
const STOPPING: &str = r"\[..:..:..\] \[Server thread/INFO\]: Stopping singleplayer server as player logged out";

const MATCHES: [&str; 2] = [
    STARTING,
    STOPPING,
];

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
        let port = (&line[43..]).parse::<u16>().unwrap();
        return MatchResult::StartedHosting(port);
    } else if matches.matched(1) {
        return MatchResult::StoppedHosting
    }
    MatchResult::Nothing
}

/*
Reads a single session of a client.
Session is defined as the minecraft client being open.
*/
fn read_session(matcher: &RegexSet, file: File, state: &mut ClientState) -> SystemTime {
    file
    // Set client state to Online
    state.set(ClientStates::Online);
    println!("Minecraft client online.");

    for result in BufReader::new(&file).lines() {
        if let Ok(line) = result {
            let result = match_line(matcher, &line);
            match result {
                MatchResult::StartedHosting(port) => {
                    // Set client state to Hosting
                    state.set(ClientStates::Hosting(port));
                    println!("Started hosting on {}.", port);
                },
                MatchResult::StoppedHosting => {
                    // Set client state to Online
                    state.set(ClientStates::Online);
                    println!("Stopped hosting.");
                }
                _ => {},
            }
        }
    };
    // Set client state to Offline
    println!("Minecraft client offline.");
    state.set(ClientStates::Offline);

    get_last_modifcation(&file)
}

/*
Returns the last time a file was modified.
*/
fn get_last_modifcation(file: &File) -> SystemTime {
    file.metadata().unwrap().modified().unwrap()
}

/*
Tracks logs and updates client state based on that.
*/
pub fn track_logs(logs_path: &String, state: &mut ClientState) {
    let sleep_time = Duration::new(LOG_CHECK_FREQUENCY.0, LOG_CHECK_FREQUENCY.1);
    let matcher = RegexSet::new(MATCHES).unwrap();
    let mut last_modification = SystemTime::now();
    loop {
        let file = File::open(logs_path).unwrap();
        if last_modification < get_last_modifcation(&file) {
            last_modification = read_session(&matcher, file, state);
        }
        sleep(sleep_time);
    }
}