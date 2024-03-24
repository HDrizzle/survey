// For logging client checkins

use std::{time::{Instant, Duration, SystemTime, UNIX_EPOCH}, sync::mpsc::Receiver, fs};

use crate::LOG_FILE;

pub fn log_main_loop(reciever: Receiver<String>) {
    loop {
        match reciever.recv() {
            Ok(req) => log(req),
            Err(e) => println!("mpsc::Receiver error: {}", e.to_string())
        }
    }
}

fn log(req: String) {
    // Get current TS
    let line = format!("{} {}\n", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), req);
    // Save line
    let mut file = fs::read_to_string(LOG_FILE).unwrap();
    file += &line;
    fs::write(LOG_FILE, file).unwrap();
}