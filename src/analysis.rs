//! For analyzing the data

use std::{collections::HashMap, fs, ops::AddAssign};
use crate::LOG_FILE;

/// Main analysis fuction
pub fn main() {
    let dataset = Dataset::load();
    println!("{}", dataset.stats());
}

fn load_and_parse_lines() -> Vec<Line> {
    let raw_file = fs::read_to_string(LOG_FILE).unwrap();
    let mut out = Vec::<Line>::new();
    for line_raw in raw_file.split('\n') {
        if line_raw.len() == 1 {continue;}
        out.push(Line::parse(line_raw.to_owned()));
    }
    // Done
    out
}

struct Dataset {
    clients: HashMap<u128, Client>,
    invalid_lines: Vec<String>
}

impl Dataset {
    pub fn load() -> Self {
        let lines = load_and_parse_lines();
        let mut invalid_lines = Vec::<String>::new();
        let mut clients = HashMap::<u128, Client>::new();
        for line in lines {
            match line {
                Line::Valid{ts: _, client_id, status: _} => match clients.get_mut(&client_id) {
                    Some(client) => client.lines.push(line),
                    None => {clients.insert(client_id, Client{lines: vec![line]});}
                },
                Line::Invalid(content) => invalid_lines.push(content)
            }
        }
        // Done
        Self {
            clients,
            invalid_lines
        }
    }
    pub fn stats(&self) -> String {
        let mut focus_times = Vec::<u32>::new();
        for (_, client) in &self.clients {
            focus_times.push(client.focus_time());
        }
        format!(
            "Number of respondants: {}\nIn-focus time (secs): {} (mean), {} (median)\nNumber of invalid response lines: {}",
            self.clients.len(),
            mean(&focus_times),
            median(&focus_times),
            self.invalid_lines.len()
        )
    }
}

struct Client {
    pub lines: Vec<Line>
}

impl Client {
    pub fn focus_time(&self) -> u32 {
        let mut total: u64 = 0;
        let mut begin_t: u64 = 0;
        let mut prev_focus = false;
        for line in &self.lines {
            match line {
                Line::Valid{ts, client_id: _, status} => {
                    let curr_focus = status != &0_u8;
                    // If focus changes
                    match (prev_focus, curr_focus) {
                        (false, false) => {},
                        (false, true) => {
                            begin_t = *ts;
                        },
                        (true, false) => {
                            total += ts - begin_t;
                        }
                        (true, true) => {}
                    }
                    // Update prev focus
                    prev_focus = curr_focus;
                },
                Line::Invalid(_) => panic!("Client should not contain invalid lines")
            }
        }
        // Done
        total as u32
    }
}

enum Line{
    Valid{
        ts: u64,
        client_id: u128,
        status: u8
    },
    Invalid(String)
}

impl Line {
    pub fn parse(raw: String) -> Self {
        let tokens: Vec<&str> = raw.split(" ").collect();
        if tokens.len() == 3 {
            let ts_res = tokens[0].parse::<u64>();
            let client_id_res = tokens[1].parse::<u128>();
            let status_res = tokens[2].parse::<u8>();
            if ts_res.is_ok() && client_id_res.is_ok() && status_res.is_ok() {
                Self::Valid{ts: ts_res.unwrap(), client_id: client_id_res.unwrap(), status: status_res.unwrap()}
            }
            else {
                Self::Invalid(raw)
            }
        }
        else {
            Self::Invalid(raw)
        }
    }
}

fn sum<T>(in_: &Vec<T>) -> T where T: AddAssign + From<u8> + Copy {
    let mut out: T = 0_u8.into();
    for n in in_ {
        out += *n;
    }
    // Done
    out
}

fn mean(in_: &Vec<u32>) -> f32 {
    assert!(in_.len() > 0);
    let sum: u32 = sum(in_);
    let sum_f: f32 = sum as f32;
    sum_f / (in_.len() as f32)
}

fn median(in_: &Vec<u32>) -> f32 {
    assert!(in_.len() > 0);
    let i_lower: usize = in_.len() / 2;
    if in_.len() % 2 == 0 && in_.len() > 1 {
        (in_[i_lower] + in_[i_lower + 1]) as f32 / 2.0
    }
    else {
        in_[i_lower] as f32
    }
}